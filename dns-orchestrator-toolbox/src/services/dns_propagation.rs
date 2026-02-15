//! DNS propagation check module.

use std::collections::HashMap;
use std::time::Instant;

use futures::future::join_all;
use tokio::time::{Duration, timeout};

use crate::error::ToolboxResult;
use crate::types::{
    DnsPropagationResult, DnsPropagationServer, DnsPropagationServerResult, DnsQueryType,
    PropagationStatus,
};

use super::dns::dns_lookup;

/// DNS query timeout in seconds.
const QUERY_TIMEOUT_SECS: u64 = 5;

/// Return the list of global DNS servers used for propagation checks.
fn get_global_dns_servers() -> Vec<DnsPropagationServer> {
    vec![
        // North America
        DnsPropagationServer {
            name: "Google DNS".to_string(),
            ip: "8.8.8.8".to_string(),
            region: "North America".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Cloudflare DNS".to_string(),
            ip: "1.1.1.1".to_string(),
            region: "North America".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Quad9 DNS".to_string(),
            ip: "9.9.9.9".to_string(),
            region: "North America".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Level3 DNS".to_string(),
            ip: "4.2.2.2".to_string(),
            region: "North America".to_string(),
            country_code: "US".to_string(),
        },
        // Europe
        DnsPropagationServer {
            name: "Cloudflare (Europe)".to_string(),
            ip: "1.0.0.1".to_string(),
            region: "Europe".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Quad9 (Europe)".to_string(),
            ip: "149.112.112.112".to_string(),
            region: "Europe".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Google (Europe)".to_string(),
            ip: "8.8.4.4".to_string(),
            region: "Europe".to_string(),
            country_code: "EU".to_string(),
        },
        // Asia
        DnsPropagationServer {
            name: "Alibaba DNS".to_string(),
            ip: "223.5.5.5".to_string(),
            region: "Asia".to_string(),
            country_code: "CN".to_string(),
        },
        DnsPropagationServer {
            name: "Tencent DNS".to_string(),
            ip: "119.29.29.29".to_string(),
            region: "Asia".to_string(),
            country_code: "CN".to_string(),
        },
        DnsPropagationServer {
            name: "DNSPod".to_string(),
            ip: "119.28.28.28".to_string(),
            region: "Asia".to_string(),
            country_code: "CN".to_string(),
        },
        // Other
        DnsPropagationServer {
            name: "OpenDNS".to_string(),
            ip: "208.67.222.222".to_string(),
            region: "North America".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "AdGuard DNS".to_string(),
            ip: "94.140.14.14".to_string(),
            region: "Europe".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Telstra Corporation Ltd".to_string(),
            ip: "139.130.4.4".to_string(),
            region: "Oceania".to_string(),
            country_code: "AU".to_string(),
        },
    ]
}

/// Calculate consistency percentage and unique answer sets.
fn calculate_consistency(results: &[DnsPropagationServerResult]) -> (f32, Vec<String>) {
    let successful_results: Vec<_> = results
        .iter()
        .filter(|r| r.status == PropagationStatus::Success)
        .collect();

    if successful_results.is_empty() {
        return (0.0, vec![]);
    }

    // Serialize each result's record values as a sorted string
    let mut value_counts: HashMap<String, usize> = HashMap::new();

    for result in &successful_results {
        let mut values: Vec<_> = result
            .records
            .iter()
            .map(|r| {
                // Compare value and priority only, not TTL
                // TTL naturally changes over time and should not affect consistency
                if let Some(priority) = r.priority {
                    format!("{}:{}", r.value, priority)
                } else {
                    r.value.clone()
                }
            })
            .collect();
        values.sort();
        let key = values.join("|");
        *value_counts.entry(key).or_insert(0) += 1;
    }

    let total = successful_results.len();
    let max_count = value_counts.values().max().copied().unwrap_or(0);
    // usize -> f64: these are small counts of DNS server results (typically < 20),
    // well within f64's precise integer range (up to 2^52)
    #[allow(clippy::cast_precision_loss)]
    let consistency = (max_count as f64 / total as f64) * 100.0;

    let unique_values: Vec<_> = value_counts.keys().cloned().collect();

    #[allow(clippy::cast_possible_truncation)]
    // f64 -> f32: consistency is a percentage (0.0..=100.0), well within f32 range
    (consistency as f32, unique_values)
}

/// Check DNS propagation across global resolvers.
pub async fn dns_propagation_check(
    domain: &str,
    record_type: DnsQueryType,
) -> ToolboxResult<DnsPropagationResult> {
    let servers = get_global_dns_servers();
    let start_time = Instant::now();

    // Query all DNS servers concurrently
    let futures: Vec<_> = servers
        .into_iter()
        .map(|server| {
            let domain = domain.to_string();
            async move {
                let query_start = Instant::now();
                let result = timeout(
                    Duration::from_secs(QUERY_TIMEOUT_SECS),
                    dns_lookup(&domain, record_type, Some(&server.ip)),
                )
                .await;
                // u128 -> u64: elapsed millis for a DNS query will never exceed u64::MAX
                #[allow(clippy::cast_possible_truncation)]
                let elapsed = query_start.elapsed().as_millis() as u64;

                match result {
                    Ok(Ok(lookup_result)) => DnsPropagationServerResult {
                        server,
                        status: PropagationStatus::Success,
                        records: lookup_result.records,
                        error: None,
                        response_time_ms: elapsed,
                    },
                    Ok(Err(e)) => DnsPropagationServerResult {
                        server,
                        status: PropagationStatus::Error,
                        records: vec![],
                        error: Some(e.to_string()),
                        response_time_ms: elapsed,
                    },
                    Err(_) => DnsPropagationServerResult {
                        server,
                        status: PropagationStatus::Timeout,
                        records: vec![],
                        error: Some(format!("Query timeout ({QUERY_TIMEOUT_SECS}s)")),
                        response_time_ms: elapsed,
                    },
                }
            }
        })
        .collect();

    let results = join_all(futures).await;

    // Calculate consistency
    let (consistency_percentage, unique_values) = calculate_consistency(&results);

    // u128 -> u64: elapsed millis for DNS propagation check will never exceed u64::MAX
    #[allow(clippy::cast_possible_truncation)]
    let total_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(DnsPropagationResult {
        domain: domain.to_string(),
        record_type,
        results,
        total_time_ms,
        consistency_percentage,
        unique_values,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::similar_names)]
mod tests {
    use super::*;
    use crate::types::DnsLookupRecord;

    // ==================== get_global_dns_servers tests ====================

    #[test]
    fn test_get_global_dns_servers_not_empty() {
        let servers = get_global_dns_servers();
        assert!(!servers.is_empty());
        assert!(servers.len() >= 10);
    }

    #[test]
    fn test_get_global_dns_servers_valid_ips() {
        let servers = get_global_dns_servers();
        for server in &servers {
            assert!(
                server.ip.parse::<std::net::IpAddr>().is_ok(),
                "Invalid IP: {} for server {}",
                server.ip,
                server.name
            );
        }
    }

    #[test]
    fn test_get_global_dns_servers_has_regions() {
        let servers = get_global_dns_servers();
        for server in &servers {
            assert!(!server.name.is_empty());
            assert!(!server.region.is_empty());
            assert!(!server.country_code.is_empty());
        }
    }

    #[test]
    fn test_get_global_dns_servers_includes_major_providers() {
        let servers = get_global_dns_servers();
        let ips: Vec<&str> = servers.iter().map(|s| s.ip.as_str()).collect();
        assert!(ips.contains(&"8.8.8.8"), "Missing Google DNS");
        assert!(ips.contains(&"1.1.1.1"), "Missing Cloudflare DNS");
        assert!(ips.contains(&"9.9.9.9"), "Missing Quad9 DNS");
    }

    // ==================== calculate_consistency tests ====================

    fn make_server() -> DnsPropagationServer {
        DnsPropagationServer {
            name: "Test".to_string(),
            ip: "1.2.3.4".to_string(),
            region: "Test".to_string(),
            country_code: "XX".to_string(),
        }
    }

    fn make_record(value: &str) -> DnsLookupRecord {
        DnsLookupRecord {
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            value: value.to_string(),
            ttl: 300,
            priority: None,
        }
    }

    fn make_success_result(values: &[&str]) -> DnsPropagationServerResult {
        DnsPropagationServerResult {
            server: make_server(),
            status: PropagationStatus::Success,
            records: values.iter().map(|v| make_record(v)).collect(),
            error: None,
            response_time_ms: 10,
        }
    }

    fn make_error_result() -> DnsPropagationServerResult {
        DnsPropagationServerResult {
            server: make_server(),
            status: PropagationStatus::Error,
            records: vec![],
            error: Some("timeout".to_string()),
            response_time_ms: 5000,
        }
    }

    #[test]
    fn test_calculate_consistency_all_same() {
        let results = vec![
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["1.2.3.4"]),
        ];
        let (consistency, unique) = calculate_consistency(&results);
        assert!((consistency - 100.0).abs() < f32::EPSILON);
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_calculate_consistency_all_different() {
        let results = vec![
            make_success_result(&["1.1.1.1"]),
            make_success_result(&["2.2.2.2"]),
            make_success_result(&["3.3.3.3"]),
        ];
        let (consistency, unique) = calculate_consistency(&results);
        // Each has 1/3 => ~33.33%
        assert!((consistency - 33.333_336).abs() < 0.01);
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_calculate_consistency_empty() {
        let results: Vec<DnsPropagationServerResult> = vec![];
        let (consistency, unique) = calculate_consistency(&results);
        assert!((consistency - 0.0).abs() < f32::EPSILON);
        assert!(unique.is_empty());
    }

    #[test]
    fn test_calculate_consistency_no_success() {
        let results = vec![make_error_result(), make_error_result()];
        let (consistency, unique) = calculate_consistency(&results);
        assert!((consistency - 0.0).abs() < f32::EPSILON);
        assert!(unique.is_empty());
    }

    #[test]
    fn test_calculate_consistency_mixed_success_error() {
        let results = vec![
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["1.2.3.4"]),
            make_error_result(),
        ];
        let (consistency, unique) = calculate_consistency(&results);
        // 2 successes with same value => 100%
        assert!((consistency - 100.0).abs() < f32::EPSILON);
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_calculate_consistency_majority() {
        let results = vec![
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["1.2.3.4"]),
            make_success_result(&["5.6.7.8"]),
        ];
        let (consistency, unique) = calculate_consistency(&results);
        // 3/4 = 75%
        assert!((consistency - 75.0).abs() < 0.01);
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn test_calculate_consistency_multiple_records_order_independent() {
        // Same records in different order should be considered consistent
        let results = vec![
            make_success_result(&["1.1.1.1", "2.2.2.2"]),
            make_success_result(&["2.2.2.2", "1.1.1.1"]),
        ];
        let (consistency, unique) = calculate_consistency(&results);
        // Records are sorted before comparison, so these should match
        assert!((consistency - 100.0).abs() < f32::EPSILON);
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_calculate_consistency_with_priority() {
        // MX records with priority should include priority in comparison
        let mut result1 = make_success_result(&["mail.example.com"]);
        result1.records[0].priority = Some(10);
        let mut result2 = make_success_result(&["mail.example.com"]);
        result2.records[0].priority = Some(20);

        let results = vec![result1, result2];
        let (consistency, unique) = calculate_consistency(&results);
        // Different priorities => different values
        assert!((consistency - 50.0).abs() < 0.01);
        assert_eq!(unique.len(), 2);
    }

    // ==================== integration tests ====================

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_dns_propagation_check_real() {
        let result = dns_propagation_check("google.com", DnsQueryType::A).await;
        assert!(result.is_ok());
        let propagation = result.unwrap();
        assert_eq!(propagation.domain, "google.com");
        assert_eq!(propagation.record_type, DnsQueryType::A);
        assert!(!propagation.results.is_empty());
        assert!(propagation.total_time_ms > 0);
        // Google should have high consistency
        assert!(propagation.consistency_percentage > 0.0);
    }
}
