//! DNS 传播检查模块

use std::collections::HashMap;
use std::time::Instant;

use futures::future::join_all;
use tokio::time::{timeout, Duration};

use crate::error::CoreResult;
use crate::types::{DnsPropagationResult, DnsPropagationServer, DnsPropagationServerResult};

use super::dns::dns_lookup;

/// DNS 查询超时时间（秒）
const QUERY_TIMEOUT_SECS: u64 = 5;

/// 获取全球 DNS 服务器列表
fn get_global_dns_servers() -> Vec<DnsPropagationServer> {
    vec![
        // 北美
        DnsPropagationServer {
            name: "Google DNS".to_string(),
            ip: "8.8.8.8".to_string(),
            region: "美国（北美）".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Cloudflare DNS".to_string(),
            ip: "1.1.1.1".to_string(),
            region: "美国（北美）".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Quad9 DNS".to_string(),
            ip: "9.9.9.9".to_string(),
            region: "美国（北美）".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "Level3 DNS".to_string(),
            ip: "4.2.2.2".to_string(),
            region: "美国（北美）".to_string(),
            country_code: "US".to_string(),
        },
        // 欧洲
        DnsPropagationServer {
            name: "Cloudflare (欧洲)".to_string(),
            ip: "1.0.0.1".to_string(),
            region: "欧洲".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Quad9 (欧洲)".to_string(),
            ip: "149.112.112.112".to_string(),
            region: "欧洲".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Google (欧洲)".to_string(),
            ip: "8.8.4.4".to_string(),
            region: "欧洲".to_string(),
            country_code: "EU".to_string(),
        },
        // 亚洲
        DnsPropagationServer {
            name: "阿里 DNS".to_string(),
            ip: "223.5.5.5".to_string(),
            region: "中国（亚洲）".to_string(),
            country_code: "CN".to_string(),
        },
        DnsPropagationServer {
            name: "腾讯 DNS".to_string(),
            ip: "119.29.29.29".to_string(),
            region: "中国（亚洲）".to_string(),
            country_code: "CN".to_string(),
        },
        DnsPropagationServer {
            name: "DNSPod".to_string(),
            ip: "119.28.28.28".to_string(),
            region: "中国（亚洲）".to_string(),
            country_code: "CN".to_string(),
        },
        // 其他
        DnsPropagationServer {
            name: "OpenDNS".to_string(),
            ip: "208.67.222.222".to_string(),
            region: "美国".to_string(),
            country_code: "US".to_string(),
        },
        DnsPropagationServer {
            name: "AdGuard DNS".to_string(),
            ip: "94.140.14.14".to_string(),
            region: "欧洲".to_string(),
            country_code: "EU".to_string(),
        },
        DnsPropagationServer {
            name: "Telstra Corporation Ltd".to_string(),
            ip: "139.130.4.4".to_string(),
            region: "澳大利亚（大洋洲）".to_string(),
            country_code: "AU".to_string(),
        },
    ]
}

/// 计算一致性百分比和唯一值
fn calculate_consistency(results: &[DnsPropagationServerResult]) -> (f32, Vec<String>) {
    let successful_results: Vec<_> = results.iter().filter(|r| r.status == "success").collect();

    if successful_results.is_empty() {
        return (0.0, vec![]);
    }

    // 将每个结果的记录值序列化为字符串（排序后）
    let mut value_counts: HashMap<String, usize> = HashMap::new();

    for result in &successful_results {
        let mut values: Vec<_> = result
            .records
            .iter()
            .map(|r| {
                // 只比较 value 和 priority，不包含 TTL
                // TTL 随时间变化是正常的，不应影响一致性判断
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
    let consistency = (max_count as f32 / total as f32) * 100.0;

    let unique_values: Vec<_> = value_counts.keys().cloned().collect();

    (consistency, unique_values)
}

/// DNS 传播检查
pub async fn dns_propagation_check(
    domain: &str,
    record_type: &str,
) -> CoreResult<DnsPropagationResult> {
    let servers = get_global_dns_servers();
    let start_time = Instant::now();

    // 并发查询所有 DNS 服务器
    let futures: Vec<_> = servers
        .into_iter()
        .map(|server| {
            let domain = domain.to_string();
            let record_type = record_type.to_string();
            async move {
                let query_start = Instant::now();
                let result = timeout(
                    Duration::from_secs(QUERY_TIMEOUT_SECS),
                    dns_lookup(&domain, &record_type, Some(&server.ip)),
                )
                .await;
                let elapsed = query_start.elapsed().as_millis() as u64;

                match result {
                    Ok(Ok(lookup_result)) => DnsPropagationServerResult {
                        server,
                        status: "success".to_string(),
                        records: lookup_result.records,
                        error: None,
                        response_time_ms: elapsed,
                    },
                    Ok(Err(e)) => DnsPropagationServerResult {
                        server,
                        status: "error".to_string(),
                        records: vec![],
                        error: Some(e.to_string()),
                        response_time_ms: elapsed,
                    },
                    Err(_) => DnsPropagationServerResult {
                        server,
                        status: "timeout".to_string(),
                        records: vec![],
                        error: Some(format!("Query timeout ({QUERY_TIMEOUT_SECS}s)")),
                        response_time_ms: elapsed,
                    },
                }
            }
        })
        .collect();

    let results = join_all(futures).await;

    // 计算一致性
    let (consistency_percentage, unique_values) = calculate_consistency(&results);

    let total_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(DnsPropagationResult {
        domain: domain.to_string(),
        record_type: record_type.to_string(),
        results,
        total_time_ms,
        consistency_percentage,
        unique_values,
    })
}
