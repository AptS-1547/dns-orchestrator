//! WHOIS lookup module.

use regex::Regex;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::error::{ToolboxError, ToolboxResult};
use crate::types::{WhoisResult, WhoisSource};

/// Perform a WHOIS lookup for a domain.
pub async fn whois_lookup(domain: &str, whois_servers: &str) -> ToolboxResult<WhoisResult> {
    let whois = WhoIs::from_string(whois_servers).map_err(|e| {
        ToolboxError::NetworkError(format!("Failed to initialize WHOIS client: {e}"))
    })?;

    let options = WhoIsLookupOptions::from_string(domain)
        .map_err(|e| ToolboxError::ValidationError(format!("Invalid domain: {e}")))?;

    let raw = whois
        .lookup_async(options)
        .await
        .map_err(|e| ToolboxError::NetworkError(format!("WHOIS query failed: {e}")))?;

    Ok(parse_whois_response(domain, &raw))
}

/// Parse structured fields from a raw WHOIS response.
fn parse_whois_response(domain: &str, raw: &str) -> WhoisResult {
    WhoisResult {
        domain: domain.to_string(),
        registrar: extract_field(
            raw,
            &[
                r"(?i)Registrar:\s*(.+)",
                r"(?i)Registrar Name:\s*(.+)",
                r"(?i)Sponsoring Registrar:\s*(.+)",
            ],
        ),
        creation_date: extract_field(
            raw,
            &[
                r"(?i)Creation Date:\s*(.+)",
                r"(?i)Created Date:\s*(.+)",
                r"(?i)Created:\s*(.+)",
                r"(?i)Registration Time:\s*(.+)",
                r"(?i)Registration Date:\s*(.+)",
            ],
        ),
        expiration_date: extract_field(
            raw,
            &[
                r"(?i)Expir(?:y|ation) Date:\s*(.+)",
                r"(?i)Registry Expiry Date:\s*(.+)",
                r"(?i)Expiration Time:\s*(.+)",
                r"(?i)paid-till:\s*(.+)",
            ],
        ),
        updated_date: extract_field(
            raw,
            &[
                r"(?i)Updated Date:\s*(.+)",
                r"(?i)Last Updated:\s*(.+)",
                r"(?i)Last Modified:\s*(.+)",
            ],
        ),
        name_servers: extract_name_servers(raw),
        status: extract_status(raw),
        raw: raw.to_string(),
        source: WhoisSource::Whois,
    }
}

/// Try multiple regex patterns and return the first match.
fn extract_field(text: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        match Regex::new(pattern) {
            Ok(re) => {
                if let Some(caps) = re.captures(text)
                    && let Some(m) = caps.get(1)
                {
                    let value = m.as_str().trim().to_string();
                    if !value.is_empty() {
                        return Some(value);
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to compile WHOIS regex pattern '{pattern}': {e}");
            }
        }
    }
    None
}

/// Extract name servers from WHOIS text.
fn extract_name_servers(text: &str) -> Vec<String> {
    let mut servers = Vec::new();
    let patterns = [
        r"(?i)Name Server:\s*(.+)",
        r"(?i)nserver:\s*(.+)",
        r"(?i)DNS:\s*(.+)",
    ];

    for pattern in patterns {
        match Regex::new(pattern) {
            Ok(re) => {
                for caps in re.captures_iter(text) {
                    if let Some(m) = caps.get(1) {
                        let server = m.as_str().trim().to_lowercase();
                        if !server.is_empty() && !servers.contains(&server) {
                            servers.push(server);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to compile WHOIS regex pattern '{pattern}': {e}");
            }
        }
    }

    servers
}

/// Extract domain status codes from WHOIS text.
fn extract_status(text: &str) -> Vec<String> {
    let mut statuses = Vec::new();
    let patterns = [
        r"(?i)Domain Status:\s*(.+)",
        r"(?i)Status:\s*(.+)",
        r"(?i)state:\s*(.+)",
    ];

    for pattern in patterns {
        match Regex::new(pattern) {
            Ok(re) => {
                for caps in re.captures_iter(text) {
                    if let Some(m) = caps.get(1) {
                        let raw = m.as_str().trim();
                        // Take only the first token (strip trailing URL or description)
                        let status = raw.split_whitespace().next().unwrap_or("").to_string();
                        if !status.is_empty() && !statuses.contains(&status) {
                            statuses.push(status);
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to compile WHOIS regex pattern '{pattern}': {e}");
            }
        }
    }

    statuses
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ==================== extract_field tests ====================

    #[test]
    fn test_extract_field_basic() {
        let text = "Registrar: Example Registrar Inc.";
        let result = extract_field(text, &[r"(?i)Registrar:\s*(.+)"]);
        assert_eq!(result, Some("Example Registrar Inc.".to_string()));
    }

    #[test]
    fn test_extract_field_case_insensitive() {
        let text = "registrar: Lower Case Registrar";
        let result = extract_field(text, &[r"(?i)Registrar:\s*(.+)"]);
        assert_eq!(result, Some("Lower Case Registrar".to_string()));
    }

    #[test]
    fn test_extract_field_multiple_patterns() {
        let text = "Sponsoring Registrar: Fallback Registrar";
        let result = extract_field(
            text,
            &[
                r"(?i)Registrar:\s*(.+)",
                r"(?i)Sponsoring Registrar:\s*(.+)",
            ],
        );
        assert_eq!(result, Some("Fallback Registrar".to_string()));
    }

    #[test]
    fn test_extract_field_first_pattern_wins() {
        let text = "Registrar: First\nSponsoring Registrar: Second";
        let result = extract_field(
            text,
            &[
                r"(?i)Registrar:\s*(.+)",
                r"(?i)Sponsoring Registrar:\s*(.+)",
            ],
        );
        assert_eq!(result, Some("First".to_string()));
    }

    #[test]
    fn test_extract_field_no_match() {
        let text = "Nothing here";
        let result = extract_field(text, &[r"(?i)Registrar:\s*(.+)"]);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_field_empty_value() {
        let text = "Registrar: ";
        let result = extract_field(text, &[r"(?i)Registrar:\s*(.*)"]);
        assert_eq!(result, None);
    }

    // ==================== extract_name_servers tests ====================

    #[test]
    fn test_extract_name_servers_basic() {
        let text = "Name Server: ns1.example.com\nName Server: ns2.example.com";
        let result = extract_name_servers(text);
        assert_eq!(result, vec!["ns1.example.com", "ns2.example.com"]);
    }

    #[test]
    fn test_extract_name_servers_lowercased() {
        let text = "Name Server: NS1.EXAMPLE.COM";
        let result = extract_name_servers(text);
        assert_eq!(result, vec!["ns1.example.com"]);
    }

    #[test]
    fn test_extract_name_servers_dedup() {
        let text = "Name Server: ns1.example.com\nName Server: NS1.EXAMPLE.COM";
        let result = extract_name_servers(text);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_extract_name_servers_nserver_format() {
        let text = "nserver: ns1.example.com\nnserver: ns2.example.com";
        let result = extract_name_servers(text);
        assert_eq!(result, vec!["ns1.example.com", "ns2.example.com"]);
    }

    #[test]
    fn test_extract_name_servers_empty() {
        let text = "No name servers here";
        let result = extract_name_servers(text);
        assert!(result.is_empty());
    }

    // ==================== extract_status tests ====================

    #[test]
    fn test_extract_status_domain_status() {
        let text = "Domain Status: clientTransferProhibited https://icann.org/epp#clientTransferProhibited";
        let result = extract_status(text);
        assert_eq!(result, vec!["clientTransferProhibited"]);
    }

    #[test]
    fn test_extract_status_multiple() {
        let text = "Domain Status: clientTransferProhibited https://example\nDomain Status: clientDeleteProhibited https://example";
        let result = extract_status(text);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&"clientTransferProhibited".to_string()));
        assert!(result.contains(&"clientDeleteProhibited".to_string()));
    }

    #[test]
    fn test_extract_status_dedup() {
        let text = "Domain Status: active\nDomain Status: active";
        let result = extract_status(text);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_extract_status_state_format() {
        let text = "state: REGISTERED, DELEGATED";
        let result = extract_status(text);
        assert_eq!(result, vec!["REGISTERED,"]);
    }

    #[test]
    fn test_extract_status_empty() {
        let text = "Nothing here";
        let result = extract_status(text);
        assert!(result.is_empty());
    }

    // ==================== parse_whois_response tests ====================

    #[test]
    fn test_parse_whois_response_full() {
        let raw = r"Domain Name: EXAMPLE.COM
Registrar: Example Registrar Inc.
Creation Date: 1995-08-14T04:00:00Z
Registry Expiry Date: 2024-08-13T04:00:00Z
Updated Date: 2023-08-14T07:01:44Z
Name Server: A.IANA-SERVERS.NET
Name Server: B.IANA-SERVERS.NET
Domain Status: clientDeleteProhibited https://icann.org
Domain Status: clientTransferProhibited https://icann.org";

        let result = parse_whois_response("example.com", raw);
        assert_eq!(result.domain, "example.com");
        assert_eq!(result.registrar, Some("Example Registrar Inc.".to_string()));
        assert_eq!(
            result.creation_date,
            Some("1995-08-14T04:00:00Z".to_string())
        );
        assert_eq!(
            result.expiration_date,
            Some("2024-08-13T04:00:00Z".to_string())
        );
        assert_eq!(result.name_servers.len(), 2);
        assert_eq!(result.status.len(), 2);
        assert_eq!(result.raw, raw);
    }

    #[test]
    fn test_parse_whois_response_empty() {
        let result = parse_whois_response("unknown.tld", "");
        assert_eq!(result.domain, "unknown.tld");
        assert!(result.registrar.is_none());
        assert!(result.creation_date.is_none());
        assert!(result.expiration_date.is_none());
        assert!(result.updated_date.is_none());
        assert!(result.name_servers.is_empty());
        assert!(result.status.is_empty());
    }

    #[test]
    fn test_parse_whois_response_cn_format() {
        let raw = r"Registration Time: 2003-03-17 12:20:05
Expiration Time: 2026-03-17 12:48:36
Sponsoring Registrar: Alibaba Cloud Computing
Name Server: ns1.example.cn
Name Server: ns2.example.cn
Status: clientTransferProhibited";

        let result = parse_whois_response("example.cn", raw);
        assert_eq!(
            result.registrar,
            Some("Alibaba Cloud Computing".to_string())
        );
        assert!(result.creation_date.is_some());
        assert!(result.expiration_date.is_some());
        assert_eq!(result.name_servers.len(), 2);
    }

    #[test]
    fn test_parse_whois_response_ru_format() {
        let raw = r"nserver: ns1.example.ru
nserver: ns2.example.ru
state: REGISTERED, DELEGATED
paid-till: 2025-12-01T00:00:00Z
Created: 2000-01-01";

        let result = parse_whois_response("example.ru", raw);
        assert!(result.creation_date.is_some());
        assert!(result.expiration_date.is_some());
        assert_eq!(result.name_servers.len(), 2);
    }

    // ==================== integration tests ====================

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_whois_lookup_real() {
        let whois_servers = include_str!("whois_servers.json");
        let result = whois_lookup("google.com", whois_servers).await;
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.domain, "google.com");
        assert!(info.registrar.is_some());
        assert!(!info.name_servers.is_empty());
    }
}
