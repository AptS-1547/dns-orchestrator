//! Public types returned by toolbox operations.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// DNS query type for lookup and propagation operations.
///
/// Includes all supported record types plus [`All`](Self::All) to query every
/// type at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsQueryType {
    /// IPv4 address record.
    A,
    /// IPv6 address record.
    Aaaa,
    /// Canonical name (alias) record.
    Cname,
    /// Mail exchange record.
    Mx,
    /// Text record.
    Txt,
    /// Name server record.
    Ns,
    /// Start of authority record.
    Soa,
    /// Service locator record.
    Srv,
    /// Certificate Authority Authorization record.
    Caa,
    /// Pointer record (reverse DNS).
    Ptr,
    /// Query all supported record types.
    All,
}

impl fmt::Display for DnsQueryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::Aaaa => write!(f, "AAAA"),
            Self::Cname => write!(f, "CNAME"),
            Self::Mx => write!(f, "MX"),
            Self::Txt => write!(f, "TXT"),
            Self::Ns => write!(f, "NS"),
            Self::Soa => write!(f, "SOA"),
            Self::Srv => write!(f, "SRV"),
            Self::Caa => write!(f, "CAA"),
            Self::Ptr => write!(f, "PTR"),
            Self::All => write!(f, "ALL"),
        }
    }
}

impl FromStr for DnsQueryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "A" => Ok(Self::A),
            "AAAA" => Ok(Self::Aaaa),
            "CNAME" => Ok(Self::Cname),
            "MX" => Ok(Self::Mx),
            "TXT" => Ok(Self::Txt),
            "NS" => Ok(Self::Ns),
            "SOA" => Ok(Self::Soa),
            "SRV" => Ok(Self::Srv),
            "CAA" => Ok(Self::Caa),
            "PTR" => Ok(Self::Ptr),
            "ALL" => Ok(Self::All),
            _ => Err(format!("Unsupported DNS query type: {s}")),
        }
    }
}

/// WHOIS query result with parsed registration fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoisResult {
    /// The queried domain name.
    pub domain: String,
    /// Domain registrar (e.g. "Cloudflare, Inc.").
    pub registrar: Option<String>,
    /// Registration creation date.
    pub creation_date: Option<String>,
    /// Registration expiration date.
    pub expiration_date: Option<String>,
    /// Last updated date.
    pub updated_date: Option<String>,
    /// Authoritative name servers.
    pub name_servers: Vec<String>,
    /// EPP status codes.
    pub status: Vec<String>,
    /// Raw WHOIS response text.
    pub raw: String,
}

/// A single DNS record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupRecord {
    /// Record type (e.g. `"A"`, `"MX"`, `"CNAME"`).
    pub record_type: String,
    /// Record name (owner).
    pub name: String,
    /// Record value / rdata.
    ///
    /// Notes:
    /// - For `NS`, `CNAME`, `MX`, `SRV`, `SOA`, and `PTR` records, trailing dots are removed from
    ///   domain names.
    /// - For some record types the value is returned in a human-readable, space-separated form:
    ///   - `SOA`: `mname rname serial refresh retry expire minimum`
    ///   - `SRV`: `weight port target`
    ///   - `CAA`: `flags tag "value"`
    pub value: String,
    /// Time-to-live in seconds.
    pub ttl: u32,
    /// Priority (MX / SRV records only).
    pub priority: Option<u16>,
}

/// Result of a DNS lookup, including the nameserver that answered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupResult {
    /// DNS resolver used for this query.
    ///
    /// When a custom nameserver is provided, this will be that IP address.
    /// Otherwise, this is a best-effort, human-readable label for the system DNS configuration.
    pub nameserver: String,
    /// Returned records.
    pub records: Vec<DnsLookupRecord>,
}

/// Geolocation data for a single IP address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpGeoInfo {
    /// IP address.
    pub ip: String,
    /// `"IPv4"` or `"IPv6"`.
    pub ip_version: String,
    /// Country name.
    pub country: Option<String>,
    /// ISO country code.
    pub country_code: Option<String>,
    /// Region / province / state.
    pub region: Option<String>,
    /// City name.
    pub city: Option<String>,
    /// Latitude.
    pub latitude: Option<f64>,
    /// Longitude.
    pub longitude: Option<f64>,
    /// IANA timezone identifier.
    pub timezone: Option<String>,
    /// Internet Service Provider.
    pub isp: Option<String>,
    /// Organisation.
    pub org: Option<String>,
    /// Autonomous System Number (e.g. `"AS13335"`).
    pub asn: Option<String>,
    /// AS name.
    pub as_name: Option<String>,
}

/// IP geolocation result -- may contain multiple IPs when a domain is queried.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpLookupResult {
    /// Original query (IP address or domain).
    pub query: String,
    /// Whether the query was a domain name (as opposed to a raw IP).
    pub is_domain: bool,
    /// Geolocation results, one per resolved IP.
    pub results: Vec<IpGeoInfo>,
}

/// SSL/TLS certificate details.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCertInfo {
    /// Primary domain name derived from the certificate (CN or first SAN entry).
    ///
    /// This may differ from the queried host in [`SslCheckResult::domain`].
    pub domain: String,
    /// Certificate issuer (e.g. `"CN=R11, O=Let's Encrypt"`).
    pub issuer: String,
    /// Certificate subject.
    pub subject: String,
    /// Not-before date (RFC 3339 / ISO 8601, UTC).
    pub valid_from: String,
    /// Not-after date (RFC 3339 / ISO 8601, UTC).
    pub valid_to: String,
    /// Days until expiration (negative if expired).
    pub days_remaining: i64,
    /// Whether the certificate has expired.
    pub is_expired: bool,
    /// Whether the certificate is valid for the queried domain.
    pub is_valid: bool,
    /// Subject Alternative Names.
    pub san: Vec<String>,
    /// Certificate serial number (hex).
    pub serial_number: String,
    /// Signature algorithm (e.g. `"SHA256withRSA"`).
    pub signature_algorithm: String,
    /// Certificate chain from leaf to root.
    pub certificate_chain: Vec<CertChainItem>,
}

/// SSL/TLS connection status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionStatus {
    Https,
    Http,
    Failed,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Https => write!(f, "https"),
            Self::Http => write!(f, "http"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// SSL connection check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCheckResult {
    /// Queried domain.
    pub domain: String,
    /// Port that was checked.
    pub port: u16,
    /// Connection status.
    pub connection_status: ConnectionStatus,
    /// Certificate info (present only when the connection succeeded over HTTPS).
    pub cert_info: Option<SslCertInfo>,
    /// Error message when the connection failed.
    pub error: Option<String>,
}

/// A single entry in the certificate chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertChainItem {
    /// Certificate subject.
    pub subject: String,
    /// Certificate issuer.
    pub issuer: String,
    /// Whether this is a CA certificate.
    pub is_ca: bool,
}

/// HTTP request method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
}

/// An HTTP header name/value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    /// Header name.
    pub name: String,
    /// Header value.
    pub value: String,
}

/// Parameters for an HTTP header check request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeaderCheckRequest {
    /// Target URL.
    pub url: String,
    /// HTTP method to use.
    pub method: HttpMethod,
    /// Custom request headers.
    pub custom_headers: Vec<HttpHeader>,
    /// Request body (POST / PUT / PATCH only).
    pub body: Option<String>,
    /// Body content-type.
    pub content_type: Option<String>,
}

/// Status of a security header check.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SecurityHeaderStatus {
    Good,
    Warning,
    Missing,
}

/// Security header analysis for a single header.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityHeaderAnalysis {
    /// Header name (e.g. `"strict-transport-security"`).
    pub name: String,
    /// Whether the header is present in the response.
    pub present: bool,
    /// Header value if present.
    pub value: Option<String>,
    /// Check status.
    pub status: SecurityHeaderStatus,
    /// Actionable recommendation.
    pub recommendation: Option<String>,
}

/// Result of an HTTP header check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeaderCheckResult {
    /// Requested URL.
    pub url: String,
    /// HTTP status code.
    pub status_code: u16,
    /// HTTP status text (e.g. `"OK"`).
    pub status_text: String,
    /// Round-trip time in milliseconds.
    pub response_time_ms: u64,
    /// All response headers.
    pub headers: Vec<HttpHeader>,
    /// Per-header security analysis.
    pub security_analysis: Vec<SecurityHeaderAnalysis>,
    /// Response body length in bytes (computed).
    pub content_length: Option<u64>,
    /// Reconstructed raw request for debugging.
    pub raw_request: String,
    /// Reconstructed raw response for debugging.
    pub raw_response: String,
}

/// Metadata for a DNS propagation check server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationServer {
    /// Human-readable name (e.g. `"Google DNS"`).
    pub name: String,
    /// Server IP address.
    pub ip: String,
    /// Geographic region.
    pub region: String,
    /// ISO country code (or a region code like `"EU"` for shared resolvers).
    pub country_code: String,
}

/// DNS propagation query status for a single server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PropagationStatus {
    Success,
    Timeout,
    Error,
}

/// Result from a single DNS propagation server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationServerResult {
    /// Server that was queried.
    pub server: DnsPropagationServer,
    /// Query status.
    pub status: PropagationStatus,
    /// Records returned on success.
    pub records: Vec<DnsLookupRecord>,
    /// Error message on failure.
    pub error: Option<String>,
    /// Query round-trip time in milliseconds.
    pub response_time_ms: u64,
}

/// DNS propagation check result across multiple global resolvers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsPropagationResult {
    /// Queried domain.
    pub domain: String,
    /// Queried record type.
    pub record_type: DnsQueryType,
    /// Per-server results.
    pub results: Vec<DnsPropagationServerResult>,
    /// Total wall-clock time in milliseconds.
    pub total_time_ms: u64,
    /// Percentage of servers returning consistent answers (0-100).
    pub consistency_percentage: f32,
    /// Distinct answer values observed across all servers.
    pub unique_values: Vec<String>,
}

/// A DNSSEC DNSKEY record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnskeyRecord {
    /// Flags (256 = ZSK, 257 = KSK).
    pub flags: u16,
    /// Protocol (always 3).
    pub protocol: u8,
    /// Algorithm number.
    pub algorithm: u8,
    /// Human-readable algorithm name.
    pub algorithm_name: String,
    /// Base64-encoded public key.
    pub public_key: String,
    /// Computed key tag.
    pub key_tag: u16,
    /// `"ZSK"` or `"KSK"`.
    pub key_type: String,
}

/// A DNSSEC DS (Delegation Signer) record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DsRecord {
    /// Key tag of the referenced DNSKEY.
    pub key_tag: u16,
    /// Algorithm number.
    pub algorithm: u8,
    /// Human-readable algorithm name.
    pub algorithm_name: String,
    /// Digest type (1 = SHA-1, 2 = SHA-256, 4 = SHA-384).
    pub digest_type: u8,
    /// Human-readable digest type name.
    pub digest_type_name: String,
    /// Hex-encoded digest.
    pub digest: String,
}

/// A DNSSEC RRSIG (Resource Record Signature) record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RrsigRecord {
    /// Record type covered by this signature.
    pub type_covered: String,
    /// Algorithm number.
    pub algorithm: u8,
    /// Human-readable algorithm name.
    pub algorithm_name: String,
    /// Number of labels in the owner name.
    pub labels: u8,
    /// Original TTL of the covered `RRset`.
    pub original_ttl: u32,
    /// Signature expiration (RFC 3339 / ISO 8601, UTC).
    pub signature_expiration: String,
    /// Signature inception (RFC 3339 / ISO 8601, UTC).
    pub signature_inception: String,
    /// Key tag of the signing DNSKEY.
    pub key_tag: u16,
    /// Signer's domain name.
    pub signer_name: String,
    /// Base64-encoded signature data.
    pub signature: String,
}

/// DNSSEC validation status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DnssecValidationStatus {
    Secure,
    Insecure,
    Bogus,
    Indeterminate,
}

impl std::fmt::Display for DnssecValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Secure => write!(f, "secure"),
            Self::Insecure => write!(f, "insecure"),
            Self::Bogus => write!(f, "bogus"),
            Self::Indeterminate => write!(f, "indeterminate"),
        }
    }
}

/// DNSSEC validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnssecResult {
    /// Queried domain.
    pub domain: String,
    /// Whether DNSSEC records were found.
    pub dnssec_enabled: bool,
    /// DNSKEY records.
    pub dnskey_records: Vec<DnskeyRecord>,
    /// DS records.
    pub ds_records: Vec<DsRecord>,
    /// RRSIG records.
    pub rrsig_records: Vec<RrsigRecord>,
    /// Validation status.
    pub validation_status: DnssecValidationStatus,
    /// DNS resolver used.
    pub nameserver: String,
    /// Query round-trip time in milliseconds.
    pub response_time_ms: u64,
    /// Error message if the query failed.
    pub error: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ==================== DnsQueryType tests ====================

    #[test]
    fn test_dns_query_type_from_str_all_variants() {
        let cases = [
            ("A", DnsQueryType::A),
            ("AAAA", DnsQueryType::Aaaa),
            ("CNAME", DnsQueryType::Cname),
            ("MX", DnsQueryType::Mx),
            ("TXT", DnsQueryType::Txt),
            ("NS", DnsQueryType::Ns),
            ("SOA", DnsQueryType::Soa),
            ("SRV", DnsQueryType::Srv),
            ("CAA", DnsQueryType::Caa),
            ("PTR", DnsQueryType::Ptr),
            ("ALL", DnsQueryType::All),
        ];
        for (input, expected) in cases {
            assert_eq!(input.parse::<DnsQueryType>().unwrap(), expected);
        }
    }

    #[test]
    fn test_dns_query_type_from_str_case_insensitive() {
        assert_eq!("a".parse::<DnsQueryType>().unwrap(), DnsQueryType::A);
        assert_eq!("aaaa".parse::<DnsQueryType>().unwrap(), DnsQueryType::Aaaa);
        assert_eq!(
            "Cname".parse::<DnsQueryType>().unwrap(),
            DnsQueryType::Cname
        );
        assert_eq!("all".parse::<DnsQueryType>().unwrap(), DnsQueryType::All);
        assert_eq!("sOa".parse::<DnsQueryType>().unwrap(), DnsQueryType::Soa);
    }

    #[test]
    fn test_dns_query_type_from_str_invalid() {
        assert!("INVALID".parse::<DnsQueryType>().is_err());
        assert!("".parse::<DnsQueryType>().is_err());
        assert!("HTTPS".parse::<DnsQueryType>().is_err());
    }

    #[test]
    fn test_dns_query_type_display_roundtrip() {
        let variants = [
            DnsQueryType::A,
            DnsQueryType::Aaaa,
            DnsQueryType::Cname,
            DnsQueryType::Mx,
            DnsQueryType::Txt,
            DnsQueryType::Ns,
            DnsQueryType::Soa,
            DnsQueryType::Srv,
            DnsQueryType::Caa,
            DnsQueryType::Ptr,
            DnsQueryType::All,
        ];
        for variant in variants {
            let s = variant.to_string();
            let parsed: DnsQueryType = s.parse().unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn test_dns_query_type_serde_roundtrip() {
        let variant = DnsQueryType::Aaaa;
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, "\"AAAA\"");
        let parsed: DnsQueryType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, variant);
    }

    #[test]
    fn test_dns_query_type_serde_all_variants() {
        let variants = [
            (DnsQueryType::A, "\"A\""),
            (DnsQueryType::Aaaa, "\"AAAA\""),
            (DnsQueryType::Cname, "\"CNAME\""),
            (DnsQueryType::Mx, "\"MX\""),
            (DnsQueryType::Txt, "\"TXT\""),
            (DnsQueryType::Ns, "\"NS\""),
            (DnsQueryType::Soa, "\"SOA\""),
            (DnsQueryType::Srv, "\"SRV\""),
            (DnsQueryType::Caa, "\"CAA\""),
            (DnsQueryType::Ptr, "\"PTR\""),
            (DnsQueryType::All, "\"ALL\""),
        ];
        for (variant, expected_json) in variants {
            assert_eq!(serde_json::to_string(&variant).unwrap(), expected_json);
        }
    }

    // ==================== existing tests ====================

    #[test]
    fn test_whois_result_camel_case_serialization() {
        let result = WhoisResult {
            domain: "example.com".to_string(),
            registrar: Some("Test Registrar".to_string()),
            creation_date: Some("2020-01-01".to_string()),
            expiration_date: None,
            updated_date: None,
            name_servers: vec!["ns1.example.com".to_string()],
            status: vec!["active".to_string()],
            raw: "raw data".to_string(),
        };
        let json = serde_json::to_value(&result).unwrap();
        // Verify camelCase serialization
        assert!(json.get("nameServers").is_some());
        assert!(json.get("creationDate").is_some());
        assert!(json.get("expirationDate").is_some());
        assert!(json.get("updatedDate").is_some());
        assert_eq!(json["domain"], "example.com");
        assert_eq!(json["registrar"], "Test Registrar");
    }

    #[test]
    fn test_whois_result_deserialization() {
        let json = r#"{
            "domain": "example.com",
            "registrar": null,
            "creationDate": null,
            "expirationDate": null,
            "updatedDate": null,
            "nameServers": [],
            "status": [],
            "raw": ""
        }"#;
        let result: WhoisResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.domain, "example.com");
        assert!(result.registrar.is_none());
        assert!(result.name_servers.is_empty());
    }

    #[test]
    fn test_dns_lookup_record_serialization() {
        let record = DnsLookupRecord {
            record_type: "A".to_string(),
            name: "example.com".to_string(),
            value: "93.184.216.34".to_string(),
            ttl: 300,
            priority: None,
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["recordType"], "A");
        assert!(json.get("priority").is_some()); // null but present
    }

    #[test]
    fn test_dns_lookup_record_with_priority() {
        let record = DnsLookupRecord {
            record_type: "MX".to_string(),
            name: "example.com".to_string(),
            value: "mail.example.com".to_string(),
            ttl: 300,
            priority: Some(10),
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["priority"], 10);
    }

    #[test]
    fn test_dns_lookup_result_serialization() {
        let result = DnsLookupResult {
            nameserver: "8.8.8.8".to_string(),
            records: vec![],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["nameserver"], "8.8.8.8");
        assert!(json["records"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_ip_geo_info_serialization() {
        let info = IpGeoInfo {
            ip: "1.1.1.1".to_string(),
            ip_version: "IPv4".to_string(),
            country: Some("United States".to_string()),
            country_code: Some("US".to_string()),
            region: Some("California".to_string()),
            city: Some("Los Angeles".to_string()),
            latitude: Some(34.0522),
            longitude: Some(-118.2437),
            timezone: Some("America/Los_Angeles".to_string()),
            isp: Some("Cloudflare".to_string()),
            org: Some("Cloudflare Inc".to_string()),
            asn: Some("AS13335".to_string()),
            as_name: Some("Cloudflare Inc".to_string()),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["ipVersion"], "IPv4");
        assert_eq!(json["countryCode"], "US");
        assert_eq!(json["asName"], "Cloudflare Inc");
    }

    #[test]
    fn test_ssl_check_result_serialization() {
        let result = SslCheckResult {
            domain: "example.com".to_string(),
            port: 443,
            connection_status: ConnectionStatus::Https,
            cert_info: None,
            error: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["connectionStatus"], "https");
        assert_eq!(json["certInfo"], serde_json::Value::Null);
    }

    #[test]
    fn test_http_method_serialization() {
        let method = HttpMethod::GET;
        let json = serde_json::to_value(&method).unwrap();
        assert_eq!(json, "GET");

        let method = HttpMethod::POST;
        let json = serde_json::to_value(&method).unwrap();
        assert_eq!(json, "POST");
    }

    #[test]
    fn test_http_method_deserialization() {
        let method: HttpMethod = serde_json::from_str("\"DELETE\"").unwrap();
        assert!(matches!(method, HttpMethod::DELETE));
    }

    #[test]
    fn test_http_header_check_request_serialization() {
        let req = HttpHeaderCheckRequest {
            url: "https://example.com".to_string(),
            method: HttpMethod::GET,
            custom_headers: vec![HttpHeader {
                name: "Accept".to_string(),
                value: "text/html".to_string(),
            }],
            body: None,
            content_type: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["customHeaders"][0]["name"], "Accept");
        assert_eq!(json["contentType"], serde_json::Value::Null);
    }

    #[test]
    fn test_security_header_analysis_serialization() {
        let analysis = SecurityHeaderAnalysis {
            name: "strict-transport-security".to_string(),
            present: true,
            value: Some("max-age=31536000".to_string()),
            status: SecurityHeaderStatus::Good,
            recommendation: None,
        };
        let json = serde_json::to_value(&analysis).unwrap();
        assert_eq!(json["name"], "strict-transport-security");
        assert_eq!(json["present"], true);
        assert_eq!(json["status"], "good");
    }

    #[test]
    fn test_dns_propagation_server_serialization() {
        let server = DnsPropagationServer {
            name: "Google DNS".to_string(),
            ip: "8.8.8.8".to_string(),
            region: "US".to_string(),
            country_code: "US".to_string(),
        };
        let json = serde_json::to_value(&server).unwrap();
        assert_eq!(json["countryCode"], "US");
    }

    #[test]
    fn test_dnssec_result_serialization() {
        let result = DnssecResult {
            domain: "example.com".to_string(),
            dnssec_enabled: true,
            dnskey_records: vec![],
            ds_records: vec![],
            rrsig_records: vec![],
            validation_status: DnssecValidationStatus::Secure,
            nameserver: "8.8.8.8".to_string(),
            response_time_ms: 42,
            error: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["dnssecEnabled"], true);
        assert_eq!(json["validationStatus"], "secure");
        assert_eq!(json["responseTimeMs"], 42);
        assert_eq!(json["dnskeyRecords"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_dnskey_record_serialization() {
        let record = DnskeyRecord {
            flags: 257,
            protocol: 3,
            algorithm: 13,
            algorithm_name: "ECDSAP256SHA256".to_string(),
            public_key: "base64data".to_string(),
            key_tag: 12345,
            key_type: "KSK".to_string(),
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["flags"], 257);
        assert_eq!(json["keyTag"], 12345);
        assert_eq!(json["keyType"], "KSK");
        assert_eq!(json["algorithmName"], "ECDSAP256SHA256");
        assert_eq!(json["publicKey"], "base64data");
    }

    #[test]
    fn test_ds_record_serialization() {
        let record = DsRecord {
            key_tag: 12345,
            algorithm: 8,
            algorithm_name: "RSA/SHA-256".to_string(),
            digest_type: 2,
            digest_type_name: "SHA-256".to_string(),
            digest: "abcdef1234".to_string(),
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["digestType"], 2);
        assert_eq!(json["digestTypeName"], "SHA-256");
    }

    #[test]
    fn test_cert_chain_item_serialization() {
        let item = CertChainItem {
            subject: "CN=example.com".to_string(),
            issuer: "CN=Let's Encrypt".to_string(),
            is_ca: false,
        };
        let json = serde_json::to_value(&item).unwrap();
        assert_eq!(json["isCa"], false);
    }

    #[test]
    fn test_roundtrip_whois_result() {
        let original = WhoisResult {
            domain: "test.com".to_string(),
            registrar: Some("Reg Inc".to_string()),
            creation_date: Some("2020-01-01".to_string()),
            expiration_date: Some("2025-01-01".to_string()),
            updated_date: Some("2023-06-15".to_string()),
            name_servers: vec!["ns1.test.com".to_string(), "ns2.test.com".to_string()],
            status: vec!["clientTransferProhibited".to_string()],
            raw: "raw whois data".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: WhoisResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.domain, original.domain);
        assert_eq!(deserialized.registrar, original.registrar);
        assert_eq!(deserialized.name_servers.len(), 2);
    }

    #[test]
    fn test_roundtrip_http_header_check_result() {
        let original = HttpHeaderCheckResult {
            url: "https://example.com".to_string(),
            status_code: 200,
            status_text: "OK".to_string(),
            response_time_ms: 100,
            headers: vec![HttpHeader {
                name: "content-type".to_string(),
                value: "text/html".to_string(),
            }],
            security_analysis: vec![],
            content_length: Some(1024),
            raw_request: "GET / HTTP/1.1\r\n".to_string(),
            raw_response: "HTTP/1.1 200 OK\r\n".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: HttpHeaderCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status_code, 200);
        assert_eq!(deserialized.headers.len(), 1);
        assert_eq!(deserialized.content_length, Some(1024));
    }
}
