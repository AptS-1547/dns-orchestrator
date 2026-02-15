//! Stateless service facade exposing all toolbox operations.
//!
//! Every method on [`ToolboxService`] is an async associated function -- no instance needed.

mod dns;
mod dns_propagation;
mod dnssec;
mod http_headers;
mod ip;
mod resolver;
mod ssl;
mod whois;

use crate::error::{ToolboxError, ToolboxResult};
use crate::types::{
    DnsLookupResult, DnsPropagationResult, DnsQueryType, DnssecResult, HttpHeaderCheckResult,
    IpLookupResult, WhoisResult,
};

/// Validate and normalise a domain name or IP address input.
///
/// Trims whitespace, passes through valid IP addresses unchanged, converts
/// internationalised domain names (IDN) to ASCII via IDNA 2008, and rejects
/// empty or overlong inputs.
fn validate_domain(domain: &str) -> ToolboxResult<String> {
    let domain = domain.trim();
    if domain.is_empty() {
        return Err(ToolboxError::ValidationError(
            "Domain name is required".to_string(),
        ));
    }
    // If it's a valid IP address, pass through without IDNA processing.
    if domain.parse::<std::net::IpAddr>().is_ok() {
        return Ok(domain.to_string());
    }
    // IDNA processing: converts Unicode labels to Punycode and validates.
    let ascii_domain = idna::domain_to_ascii_strict(domain)
        .map_err(|_| ToolboxError::ValidationError(format!("Invalid domain name: {domain}")))?;
    if ascii_domain.len() > 253 {
        return Err(ToolboxError::ValidationError(format!(
            "Domain name exceeds maximum length of 253 characters (got {})",
            ascii_domain.len()
        )));
    }
    Ok(ascii_domain)
}

/// Embedded WHOIS server mapping (TLD -> server).
const WHOIS_SERVERS: &str = include_str!("whois_servers.json");

/// Entry point for all network diagnostic operations.
///
/// All methods are stateless associated functions -- call them directly on the type.
///
/// ```rust,no_run
/// use dns_orchestrator_toolbox::{ToolboxService, DnsQueryType};
/// # async fn demo() -> dns_orchestrator_toolbox::ToolboxResult<()> {
/// let dns = ToolboxService::dns_lookup("example.com", DnsQueryType::A, None).await?;
/// # Ok(())
/// # }
/// ```
pub struct ToolboxService;

impl ToolboxService {
    /// Query WHOIS information for a domain.
    ///
    /// Returns structured registration data (registrar, dates, name servers, status)
    /// parsed from the raw WHOIS response.
    pub async fn whois_lookup(domain: &str) -> ToolboxResult<WhoisResult> {
        let domain = validate_domain(domain)?;
        whois::whois_lookup(&domain, WHOIS_SERVERS).await
    }

    /// Resolve DNS records for a domain.
    ///
    /// Pass `None` for `nameserver` to use the system default resolver.
    ///
    /// Notes:
    /// - This is a best-effort lookup. Resolver/network errors are currently logged and result in
    ///   an empty `records` list, rather than a `ToolboxError::NetworkError`.
    pub async fn dns_lookup(
        domain: &str,
        record_type: DnsQueryType,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnsLookupResult> {
        let domain = validate_domain(domain)?;
        dns::dns_lookup(&domain, record_type, nameserver).await
    }

    /// Look up geolocation data for an IP address or domain.
    ///
    /// When given a domain, resolves A/AAAA records first and geolocates each resulting IP.
    pub async fn ip_lookup(query: &str) -> ToolboxResult<IpLookupResult> {
        ip::ip_lookup(query).await
    }

    /// Inspect the SSL/TLS certificate served by a host.
    ///
    /// Defaults to port 443 when `port` is `None`.
    /// Returns connection status (`"https"`, `"http"`, or `"failed"`) and certificate details.
    ///
    /// Most connection/TLS failures are reported via the returned `SslCheckResult` fields
    /// (`connection_status` and `error`), rather than as a `ToolboxError`.
    pub async fn ssl_check(
        domain: &str,
        port: Option<u16>,
    ) -> ToolboxResult<crate::types::SslCheckResult> {
        let domain = validate_domain(domain)?;
        ssl::ssl_check(&domain, port).await
    }

    /// Send an HTTP request and analyse the response headers.
    ///
    /// Evaluates security headers (HSTS, CSP, X-Frame-Options, etc.) and returns
    /// per-header status with recommendations.
    pub async fn http_header_check(
        request: &crate::types::HttpHeaderCheckRequest,
    ) -> ToolboxResult<HttpHeaderCheckResult> {
        http_headers::http_header_check(request).await
    }

    /// Check DNS propagation across 13 global resolvers.
    ///
    /// Returns per-server results and an overall consistency percentage.
    pub async fn dns_propagation_check(
        domain: &str,
        record_type: DnsQueryType,
    ) -> ToolboxResult<DnsPropagationResult> {
        let domain = validate_domain(domain)?;
        dns_propagation::dns_propagation_check(&domain, record_type).await
    }

    /// Validate DNSSEC deployment for a domain.
    ///
    /// Queries DNSKEY, DS, and RRSIG records. The `validation_status` field will be
    /// `"secure"`, `"insecure"`, or `"indeterminate"`.
    pub async fn dnssec_check(
        domain: &str,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnssecResult> {
        let domain = validate_domain(domain)?;
        dnssec::dnssec_check(&domain, nameserver).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::validate_domain;
    use crate::error::ToolboxError;

    #[test]
    fn test_validate_domain_normal() {
        assert_eq!(validate_domain("example.com").unwrap(), "example.com");
    }

    #[test]
    fn test_validate_domain_idn() {
        assert_eq!(validate_domain("münchen.de").unwrap(), "xn--mnchen-3ya.de");
    }

    #[test]
    fn test_validate_domain_ipv4_passthrough() {
        assert_eq!(validate_domain("1.2.3.4").unwrap(), "1.2.3.4");
    }

    #[test]
    fn test_validate_domain_ipv6_passthrough() {
        assert_eq!(validate_domain("::1").unwrap(), "::1");
        assert_eq!(
            validate_domain("2606:4700::1111").unwrap(),
            "2606:4700::1111"
        );
    }

    #[test]
    fn test_validate_domain_trims_whitespace() {
        assert_eq!(validate_domain("  example.com  ").unwrap(), "example.com");
    }

    #[test]
    fn test_validate_domain_empty() {
        assert!(matches!(
            validate_domain(""),
            Err(ToolboxError::ValidationError(_))
        ));
    }

    #[test]
    fn test_validate_domain_whitespace_only() {
        assert!(matches!(
            validate_domain("   "),
            Err(ToolboxError::ValidationError(_))
        ));
    }

    #[test]
    fn test_validate_domain_invalid() {
        assert!(matches!(
            validate_domain("not a valid domain!!!"),
            Err(ToolboxError::ValidationError(_))
        ));
    }

    #[test]
    fn test_validate_domain_max_length_253() {
        // Build a domain that is exactly 253 characters after IDNA processing.
        // "a]" labels of 63 chars each: "aaa...aaa.aaa...aaa.aaa...aaa.aaa...aaa" = 63*4 + 3 dots = 255
        // We need exactly 253, so: 63 + 1 + 63 + 1 + 63 + 1 + 59 = 251 ... let's just compute it.
        // 4 labels: 62.62.62.63 = 62+1+62+1+62+1+63 = 252. Need 253.
        // 63.63.63.61 = 63+1+63+1+63+1+61 = 253. Perfect.
        let label_a = "a".repeat(63);
        let label_b = "a".repeat(63);
        let label_c = "a".repeat(63);
        let label_d = "a".repeat(61);
        let domain = format!("{label_a}.{label_b}.{label_c}.{label_d}");
        assert_eq!(domain.len(), 253);
        // This should pass validation (all ASCII, no IDNA transformation needed)
        let result = validate_domain(&domain);
        assert!(
            result.is_ok(),
            "A 253-char domain should be valid, got: {result:?}"
        );
    }

    #[test]
    fn test_validate_domain_over_max_length() {
        // 254 characters: one more than the max
        let label_a = "a".repeat(63);
        let label_b = "a".repeat(63);
        let label_c = "a".repeat(63);
        let label_d = "a".repeat(62);
        let domain = format!("{label_a}.{label_b}.{label_c}.{label_d}");
        assert_eq!(domain.len(), 254);
        let result = validate_domain(&domain);
        assert!(
            matches!(result, Err(ToolboxError::ValidationError(_))),
            "A 254-char domain should fail validation, got: {result:?}"
        );
    }
}
