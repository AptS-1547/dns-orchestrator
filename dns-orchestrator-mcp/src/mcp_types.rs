//! Compact MCP response types optimized for LLM token efficiency.
//!
//! These types wrap the toolbox result types, stripping redundant fields
//! (raw WHOIS text, cryptographic keys, repeated records, null values)
//! before serialization to the MCP client.

use dns_orchestrator_toolbox::{
    DnsLookupRecord, DnsLookupResult, DnsPropagationResult, DnsQueryType, DnssecResult,
    IpLookupResult, PropagationStatus, WhoisResult,
};
use serde::Serialize;

use dns_orchestrator_core::types::{Account, DnsRecord, PaginatedResponse, RecordData};

fn is_empty_vec<T>(v: &[T]) -> bool {
    v.is_empty()
}

// ---------------------------------------------------------------------------
// DNS Lookup
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnsLookupRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
}

impl From<DnsLookupRecord> for McpDnsLookupRecord {
    fn from(r: DnsLookupRecord) -> Self {
        Self {
            record_type: r.record_type,
            name: r.name,
            value: r.value,
            ttl: r.ttl,
            priority: r.priority,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnsLookupResult {
    pub nameserver: String,
    pub records: Vec<McpDnsLookupRecord>,
}

impl From<DnsLookupResult> for McpDnsLookupResult {
    fn from(r: DnsLookupResult) -> Self {
        Self {
            nameserver: r.nameserver,
            records: r.records.into_iter().map(Into::into).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// WHOIS
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpWhoisResult {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registrar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_date: Option<String>,
    pub name_servers: Vec<String>,
    pub status: Vec<String>,
    // `raw` field intentionally omitted — too verbose for LLM context.
}

impl From<WhoisResult> for McpWhoisResult {
    fn from(r: WhoisResult) -> Self {
        Self {
            domain: r.domain,
            registrar: r.registrar,
            creation_date: r.creation_date,
            expiration_date: r.expiration_date,
            updated_date: r.updated_date,
            name_servers: r.name_servers,
            status: r.status,
        }
    }
}

// ---------------------------------------------------------------------------
// IP Lookup
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpIpGeoInfo {
    pub ip: String,
    pub ip_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    // latitude/longitude intentionally omitted — not useful for LLM.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_name: Option<String>,
}

impl From<dns_orchestrator_toolbox::IpGeoInfo> for McpIpGeoInfo {
    fn from(r: dns_orchestrator_toolbox::IpGeoInfo) -> Self {
        Self {
            ip: r.ip,
            ip_version: r.ip_version,
            country: r.country,
            country_code: r.country_code,
            region: r.region,
            city: r.city,
            timezone: r.timezone,
            isp: r.isp,
            org: r.org,
            asn: r.asn,
            as_name: r.as_name,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpIpLookupResult {
    pub query: String,
    pub is_domain: bool,
    pub results: Vec<McpIpGeoInfo>,
}

impl From<IpLookupResult> for McpIpLookupResult {
    fn from(r: IpLookupResult) -> Self {
        Self {
            query: r.query,
            is_domain: r.is_domain,
            results: r.results.into_iter().map(Into::into).collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// DNS Propagation
// ---------------------------------------------------------------------------

/// Per-server entry used when propagation is NOT fully consistent.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPropagationServerEntry {
    pub server: String,
    pub status: PropagationStatus,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Compact propagation result.
///
/// When all servers agree (`consistencyPercentage == 100`):
///   - `records` contains the shared records (once).
///   - `servers` lists every server as a compact string.
///   - `serverResults` is empty (omitted).
///
/// When servers disagree:
///   - `records` is empty (omitted).
///   - `servers` is empty (omitted).
///   - `serverResults` lists every server with its values / errors.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnsPropagationResult {
    pub domain: String,
    pub record_type: DnsQueryType,
    pub consistency_percentage: f32,
    pub total_time_ms: u64,
    pub unique_values: Vec<String>,
    /// Shared records — only when 100 % consistent.
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub records: Vec<McpDnsLookupRecord>,
    /// All servers as compact strings — only when 100 % consistent.
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub servers: Vec<String>,
    /// Per-server details — only when NOT 100 % consistent.
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub server_results: Vec<McpPropagationServerEntry>,
}

fn format_server(name: &str, ip: &str, country_code: &str) -> String {
    format!("{name} ({ip}, {country_code})")
}

impl From<DnsPropagationResult> for McpDnsPropagationResult {
    fn from(r: DnsPropagationResult) -> Self {
        let fully_consistent = (r.consistency_percentage - 100.0).abs() < f32::EPSILON;

        if fully_consistent {
            // Take records from the first successful server.
            let records = r
                .results
                .iter()
                .find(|s| s.status == PropagationStatus::Success)
                .map(|s| s.records.iter().cloned().map(Into::into).collect())
                .unwrap_or_default();

            let servers = r
                .results
                .iter()
                .map(|s| format_server(&s.server.name, &s.server.ip, &s.server.country_code))
                .collect();

            Self {
                domain: r.domain,
                record_type: r.record_type,
                consistency_percentage: r.consistency_percentage,
                total_time_ms: r.total_time_ms,
                unique_values: r.unique_values,
                records,
                servers,
                server_results: Vec::new(),
            }
        } else {
            let server_results = r
                .results
                .into_iter()
                .map(|s| {
                    let server =
                        format_server(&s.server.name, &s.server.ip, &s.server.country_code);
                    let values = s.records.iter().map(|rec| rec.value.clone()).collect();
                    McpPropagationServerEntry {
                        server,
                        status: s.status,
                        values,
                        error: s.error,
                    }
                })
                .collect();

            Self {
                domain: r.domain,
                record_type: r.record_type,
                consistency_percentage: r.consistency_percentage,
                total_time_ms: r.total_time_ms,
                unique_values: r.unique_values,
                records: Vec::new(),
                servers: Vec::new(),
                server_results,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DNSSEC
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnskeyRecord {
    pub key_tag: u16,
    pub algorithm_name: String,
    pub key_type: String,
    // flags, protocol, algorithm, publicKey intentionally omitted.
}

impl From<dns_orchestrator_toolbox::DnskeyRecord> for McpDnskeyRecord {
    fn from(r: dns_orchestrator_toolbox::DnskeyRecord) -> Self {
        Self {
            key_tag: r.key_tag,
            algorithm_name: r.algorithm_name,
            key_type: r.key_type,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDsRecord {
    pub key_tag: u16,
    pub algorithm_name: String,
    pub digest_type_name: String,
    // algorithm, digestType, digest intentionally omitted.
}

impl From<dns_orchestrator_toolbox::DsRecord> for McpDsRecord {
    fn from(r: dns_orchestrator_toolbox::DsRecord) -> Self {
        Self {
            key_tag: r.key_tag,
            algorithm_name: r.algorithm_name,
            digest_type_name: r.digest_type_name,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRrsigRecord {
    pub type_covered: String,
    pub algorithm_name: String,
    pub key_tag: u16,
    pub signer_name: String,
    // labels, originalTtl, signatureExpiration/Inception, signature, algorithm omitted.
}

impl From<dns_orchestrator_toolbox::RrsigRecord> for McpRrsigRecord {
    fn from(r: dns_orchestrator_toolbox::RrsigRecord) -> Self {
        Self {
            type_covered: r.type_covered,
            algorithm_name: r.algorithm_name,
            key_tag: r.key_tag,
            signer_name: r.signer_name,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnssecResult {
    pub domain: String,
    pub dnssec_enabled: bool,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub dnskey_records: Vec<McpDnskeyRecord>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub ds_records: Vec<McpDsRecord>,
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub rrsig_records: Vec<McpRrsigRecord>,
    pub validation_status: dns_orchestrator_toolbox::DnssecValidationStatus,
    pub nameserver: String,
    pub response_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<DnssecResult> for McpDnssecResult {
    fn from(r: DnssecResult) -> Self {
        Self {
            domain: r.domain,
            dnssec_enabled: r.dnssec_enabled,
            dnskey_records: r.dnskey_records.into_iter().map(Into::into).collect(),
            ds_records: r.ds_records.into_iter().map(Into::into).collect(),
            rrsig_records: r.rrsig_records.into_iter().map(Into::into).collect(),
            validation_status: r.validation_status,
            nameserver: r.nameserver,
            response_time_ms: r.response_time_ms,
            error: r.error,
        }
    }
}

// ---------------------------------------------------------------------------
// Service tools: Accounts / Domains / Records
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAccount {
    pub id: String,
    pub name: String,
    pub provider: dns_orchestrator_core::types::ProviderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<dns_orchestrator_core::types::AccountStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    // createdAt / updatedAt intentionally omitted.
}

impl From<Account> for McpAccount {
    fn from(a: Account) -> Self {
        Self {
            id: a.id,
            name: a.name,
            provider: a.provider,
            status: a.status,
            error: a.error,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppDomain {
    pub id: String,
    pub name: String,
    pub provider: dns_orchestrator_core::types::ProviderType,
    pub status: dns_orchestrator_core::types::DomainStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_count: Option<u32>,
    // accountId intentionally omitted — redundant with the query parameter.
    // metadata intentionally omitted — isFavorite/color not useful for LLM.
}

impl From<dns_orchestrator_core::types::AppDomain> for McpAppDomain {
    fn from(d: dns_orchestrator_core::types::AppDomain) -> Self {
        Self {
            id: d.id,
            name: d.name,
            provider: d.provider,
            status: d.status,
            record_count: d.record_count,
        }
    }
}

#[allow(clippy::trivially_copy_pass_by_ref, clippy::ref_option)]
fn should_skip_proxied(v: &Option<bool>) -> bool {
    !matches!(v, Some(true))
}

/// Flat DNS record — `data` is decomposed into `type` + `value`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDnsRecord {
    pub id: String,
    pub name: String,
    pub ttl: u32,
    #[serde(rename = "type")]
    pub record_type: String,
    pub value: String,
    #[serde(skip_serializing_if = "should_skip_proxied")]
    pub proxied: Option<bool>,
    // domainId intentionally omitted — redundant with the query parameter.
    // createdAt / updatedAt intentionally omitted.
}

/// Flatten `RecordData` into a (type, value) pair.
fn flatten_record_data(data: RecordData) -> (String, String) {
    match data {
        RecordData::A { address } => ("A".into(), address),
        RecordData::AAAA { address } => ("AAAA".into(), address),
        RecordData::CNAME { target } => ("CNAME".into(), target),
        RecordData::MX { priority, exchange } => ("MX".into(), format!("{priority} {exchange}")),
        RecordData::TXT { text } => ("TXT".into(), text),
        RecordData::NS { nameserver } => ("NS".into(), nameserver),
        RecordData::SRV {
            priority,
            weight,
            port,
            target,
        } => ("SRV".into(), format!("{priority} {weight} {port} {target}")),
        RecordData::CAA { flags, tag, value } => {
            ("CAA".into(), format!("{flags} {tag} \"{value}\""))
        }
    }
}

impl From<DnsRecord> for McpDnsRecord {
    fn from(r: DnsRecord) -> Self {
        let (record_type, value) = flatten_record_data(r.data);
        Self {
            id: r.id,
            name: r.name,
            ttl: r.ttl,
            record_type,
            value,
            proxied: r.proxied,
        }
    }
}

/// Generic paginated response wrapper.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPaginatedResponse<T: Serialize> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total_count: u32,
    pub has_more: bool,
}

impl<T, U> From<PaginatedResponse<T>> for McpPaginatedResponse<U>
where
    U: Serialize + From<T>,
{
    fn from(r: PaginatedResponse<T>) -> Self {
        Self {
            items: r.items.into_iter().map(Into::into).collect(),
            page: r.page,
            page_size: r.page_size,
            total_count: r.total_count,
            has_more: r.has_more,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use dns_orchestrator_toolbox::*;

    #[test]
    fn whois_omits_raw_and_null_fields() {
        let result = McpWhoisResult::from(WhoisResult {
            domain: "example.com".to_string(),
            registrar: Some("Reg Inc".to_string()),
            creation_date: None,
            expiration_date: None,
            updated_date: None,
            name_servers: vec!["ns1.example.com".to_string()],
            status: vec!["active".to_string()],
            raw: "huge legal boilerplate".to_string(),
        });
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("raw"));
        assert!(!json.contains("creationDate"));
        assert!(!json.contains("expirationDate"));
        assert!(!json.contains("updatedDate"));
        assert!(json.contains("\"registrar\":\"Reg Inc\""));
    }

    #[test]
    fn dns_lookup_omits_null_priority() {
        let result = McpDnsLookupResult::from(DnsLookupResult {
            nameserver: "8.8.8.8".to_string(),
            records: vec![DnsLookupRecord {
                record_type: "A".to_string(),
                name: "example.com".to_string(),
                value: "1.1.1.1".to_string(),
                ttl: 300,
                priority: None,
            }],
        });
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("priority"));
    }

    #[test]
    fn ip_lookup_omits_lat_lon_and_nulls() {
        let result = McpIpLookupResult::from(IpLookupResult {
            query: "1.1.1.1".to_string(),
            is_domain: false,
            results: vec![IpGeoInfo {
                ip: "1.1.1.1".to_string(),
                ip_version: "IPv4".to_string(),
                country: Some("US".to_string()),
                country_code: Some("US".to_string()),
                region: None,
                city: None,
                latitude: Some(34.0),
                longitude: Some(-118.0),
                timezone: None,
                isp: None,
                org: None,
                asn: None,
                as_name: None,
            }],
        });
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("latitude"));
        assert!(!json.contains("longitude"));
        assert!(!json.contains("region"));
        assert!(!json.contains("timezone"));
    }

    #[test]
    fn propagation_consistent_uses_compact_format() {
        let record = DnsLookupRecord {
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            value: "1.1.1.1".to_string(),
            ttl: 300,
            priority: None,
        };
        let server = |name: &str, ip: &str, cc: &str| DnsPropagationServerResult {
            server: DnsPropagationServer {
                name: name.to_string(),
                ip: ip.to_string(),
                region: "NA".to_string(),
                country_code: cc.to_string(),
            },
            status: PropagationStatus::Success,
            records: vec![record.clone()],
            error: None,
            response_time_ms: 2,
        };

        let result = McpDnsPropagationResult::from(DnsPropagationResult {
            domain: "example.com".to_string(),
            record_type: DnsQueryType::A,
            results: vec![
                server("Google DNS", "8.8.8.8", "US"),
                server("Cloudflare", "1.1.1.1", "US"),
            ],
            total_time_ms: 3,
            consistency_percentage: 100.0,
            unique_values: vec!["1.1.1.1".to_string()],
        });

        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // records appear once at top level
        assert!(v.get("records").is_some());
        assert_eq!(v["records"].as_array().unwrap().len(), 1);

        // servers are compact strings
        assert!(v.get("servers").is_some());
        let servers = v["servers"].as_array().unwrap();
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0], "Google DNS (8.8.8.8, US)");

        // no serverResults when consistent
        assert!(v.get("serverResults").is_none());
    }

    #[test]
    fn propagation_inconsistent_uses_detailed_format() {
        let result = McpDnsPropagationResult::from(DnsPropagationResult {
            domain: "example.com".to_string(),
            record_type: DnsQueryType::A,
            results: vec![
                DnsPropagationServerResult {
                    server: DnsPropagationServer {
                        name: "Google DNS".to_string(),
                        ip: "8.8.8.8".to_string(),
                        region: "NA".to_string(),
                        country_code: "US".to_string(),
                    },
                    status: PropagationStatus::Success,
                    records: vec![DnsLookupRecord {
                        record_type: "A".to_string(),
                        name: "example.com".to_string(),
                        value: "1.1.1.1".to_string(),
                        ttl: 300,
                        priority: None,
                    }],
                    error: None,
                    response_time_ms: 2,
                },
                DnsPropagationServerResult {
                    server: DnsPropagationServer {
                        name: "Alibaba DNS".to_string(),
                        ip: "223.5.5.5".to_string(),
                        region: "Asia".to_string(),
                        country_code: "CN".to_string(),
                    },
                    status: PropagationStatus::Success,
                    records: vec![DnsLookupRecord {
                        record_type: "A".to_string(),
                        name: "example.com".to_string(),
                        value: "2.2.2.2".to_string(),
                        ttl: 60,
                        priority: None,
                    }],
                    error: None,
                    response_time_ms: 5,
                },
            ],
            total_time_ms: 10,
            consistency_percentage: 50.0,
            unique_values: vec!["1.1.1.1".to_string(), "2.2.2.2".to_string()],
        });

        let json = serde_json::to_string(&result).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // no top-level records/servers
        assert!(v.get("records").is_none());
        assert!(v.get("servers").is_none());

        // serverResults present
        let sr = v["serverResults"].as_array().unwrap();
        assert_eq!(sr.len(), 2);
        assert_eq!(sr[0]["server"], "Google DNS (8.8.8.8, US)");
        assert_eq!(sr[0]["values"][0], "1.1.1.1");
        assert_eq!(sr[1]["server"], "Alibaba DNS (223.5.5.5, CN)");
    }

    #[test]
    fn dnssec_omits_crypto_data() {
        let result = McpDnssecResult::from(DnssecResult {
            domain: "example.com".to_string(),
            dnssec_enabled: true,
            dnskey_records: vec![DnskeyRecord {
                flags: 257,
                protocol: 3,
                algorithm: 13,
                algorithm_name: "ECDSAP256SHA256".to_string(),
                public_key: "mdsswUyr3DPW132mOi8V9xESWE8jTo0dxCjjnopKl+GqJxpVXckHAeF+KkxLbxILfDLUT0rAK9iUzy1L53eKGQ==".to_string(),
                key_tag: 2371,
                key_type: "KSK".to_string(),
            }],
            ds_records: vec![DsRecord {
                key_tag: 2371,
                algorithm: 13,
                algorithm_name: "ECDSAP256SHA256".to_string(),
                digest_type: 2,
                digest_type_name: "SHA-256".to_string(),
                digest: "8c91baa7819a186aaf8ea3eebbc99d5c585c1478821aea07e55af25160cced83".to_string(),
            }],
            rrsig_records: Vec::new(),
            validation_status: DnssecValidationStatus::Secure,
            nameserver: "8.8.8.8".to_string(),
            response_time_ms: 42,
            error: None,
        });

        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("publicKey"));
        assert!(!json.contains("mdsswUyr3DPW"));
        assert!(!json.contains("8c91baa7")); // raw digest value
        assert!(!json.contains("\"digest\":")); // digest field itself
        assert!(!json.contains("\"digestType\":")); // numeric digestType
        assert!(!json.contains("flags"));
        assert!(!json.contains("protocol"));
        assert!(!json.contains("rrsigRecords"));
        assert!(json.contains("digestTypeName")); // this one stays
        assert!(json.contains("\"keyTag\":2371"));
        assert!(json.contains("ECDSAP256SHA256"));
        assert!(json.contains("\"validationStatus\":\"secure\""));
    }
}
