//! RDAP (Registration Data Access Protocol) lookup module.
//!
//! Queries domain registration data via RDAP (RFC 7480-7484), which returns
//! structured JSON instead of free-form text.  Falls back gracefully so the
//! caller can retry with traditional WHOIS when RDAP is unavailable.

use std::sync::LazyLock;

use reqwest::Client;
use serde::Deserialize;
use tokio::time::timeout;

use crate::error::{ToolboxError, ToolboxResult};
use crate::types::{WhoisResult, WhoisSource};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Per-request timeout for the RDAP HTTP call.
const RDAP_REQUEST_TIMEOUT_SECS: u64 = 10;

/// Overall timeout wrapping the entire lookup (including bootstrap resolution).
const RDAP_OVERALL_TIMEOUT_SECS: u64 = 15;

// ---------------------------------------------------------------------------
// HTTP client (shared, lazy)
// ---------------------------------------------------------------------------

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .timeout(std::time::Duration::from_secs(RDAP_REQUEST_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .unwrap_or_default()
});

// ---------------------------------------------------------------------------
// IANA RDAP bootstrap data
// ---------------------------------------------------------------------------

/// Embedded IANA RDAP bootstrap snapshot (from <https://data.iana.org/rdap/dns.json>).
const BOOTSTRAP_JSON: &str = include_str!("rdap_bootstrap.json");

/// Parsed bootstrap: maps TLDs to RDAP base URLs.
#[derive(Deserialize)]
struct RdapBootstrap {
    services: Vec<(Vec<String>, Vec<String>)>,
}

static BOOTSTRAP: LazyLock<RdapBootstrap> = LazyLock::new(|| {
    serde_json::from_str(BOOTSTRAP_JSON).expect("embedded rdap_bootstrap.json must be valid")
});

// ---------------------------------------------------------------------------
// RDAP response types (private, deserialise-only)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RdapDomainResponse {
    #[serde(default)]
    ldh_name: Option<String>,
    #[serde(default)]
    events: Vec<RdapEvent>,
    #[serde(default)]
    entities: Vec<RdapEntity>,
    #[serde(default)]
    nameservers: Vec<RdapNameserver>,
    #[serde(default)]
    status: Vec<String>,
    /// Non-zero when the RDAP server returns an error object.
    #[serde(default)]
    error_code: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RdapEvent {
    #[serde(default)]
    event_action: String,
    #[serde(default)]
    event_date: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RdapEntity {
    #[serde(default)]
    roles: Vec<String>,
    /// Raw jCard (RFC 7095) array — we only need the `fn` property.
    #[serde(default)]
    vcard_array: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RdapNameserver {
    #[serde(default)]
    ldh_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Bootstrap lookup
// ---------------------------------------------------------------------------

/// Find the RDAP base URL for a given domain by extracting its TLD and
/// searching the IANA bootstrap data.
fn find_rdap_server(domain: &str) -> Option<&str> {
    // Try progressively shorter suffixes: "co.uk" first, then "uk".
    let lower = domain.to_lowercase();
    let stripped = lower.strip_suffix('.').unwrap_or(&lower);
    let labels: Vec<&str> = stripped.split('.').collect();

    for start in 0..labels.len() {
        let suffix = labels[start..].join(".");
        for (tlds, urls) in &BOOTSTRAP.services {
            if tlds.iter().any(|t| t.eq_ignore_ascii_case(&suffix)) {
                // Return the first URL, stripping a trailing slash for consistency.
                return urls.first().map(|u| u.trim_end_matches('/'));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Field extraction helpers
// ---------------------------------------------------------------------------

/// Find the first event matching a given action and return its date string.
fn find_event_date(events: &[RdapEvent], action: &str) -> Option<String> {
    events
        .iter()
        .find(|e| e.event_action.eq_ignore_ascii_case(action))
        .and_then(|e| e.event_date.clone())
}

/// Extract the registrar organisation name from the entities list.
///
/// Looks for an entity whose `roles` contains `"registrar"` and reads the `fn`
/// property from its jCard (`vcardArray`).
fn extract_registrar_name(entities: &[RdapEntity]) -> Option<String> {
    for entity in entities {
        let is_registrar = entity
            .roles
            .iter()
            .any(|r| r.eq_ignore_ascii_case("registrar"));
        if !is_registrar {
            continue;
        }

        // vcardArray layout: ["vcard", [ [prop, meta, type, value], ... ]]
        if let Some(props) = entity.vcard_array.get(1).and_then(|v| v.as_array()) {
            for prop in props {
                if let Some(arr) = prop.as_array() {
                    if arr.first().and_then(|v| v.as_str()) == Some("fn") {
                        if let Some(name) = arr.get(3).and_then(|v| v.as_str()) {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                return Some(trimmed.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Perform an RDAP lookup for a domain, with an overall timeout guard.
pub async fn rdap_lookup(domain: &str) -> ToolboxResult<WhoisResult> {
    timeout(
        std::time::Duration::from_secs(RDAP_OVERALL_TIMEOUT_SECS),
        rdap_lookup_inner(domain),
    )
    .await
    .map_err(|_| ToolboxError::NetworkError(format!("RDAP lookup timed out for {domain}")))?
}

/// Inner implementation without the overall timeout wrapper.
async fn rdap_lookup_inner(domain: &str) -> ToolboxResult<WhoisResult> {
    let base_url = find_rdap_server(domain).ok_or_else(|| {
        ToolboxError::NetworkError(format!("No RDAP server found for domain: {domain}"))
    })?;

    let url = format!("{base_url}/domain/{domain}");
    log::debug!("[RDAP] querying {url}");

    let resp = HTTP_CLIENT
        .get(&url)
        .header("Accept", "application/rdap+json")
        .send()
        .await
        .map_err(|e| ToolboxError::NetworkError(format!("RDAP request failed for {domain}: {e}")))?;

    let status = resp.status();
    let body = resp
        .text()
        .await
        .map_err(|e| ToolboxError::NetworkError(format!("Failed to read RDAP response: {e}")))?;

    if !status.is_success() {
        return Err(ToolboxError::NetworkError(format!(
            "RDAP server returned HTTP {status} for {domain}"
        )));
    }

    let rdap: RdapDomainResponse = serde_json::from_str(&body).map_err(|e| {
        ToolboxError::NetworkError(format!("Failed to parse RDAP JSON for {domain}: {e}"))
    })?;

    if let Some(code) = rdap.error_code {
        return Err(ToolboxError::NetworkError(format!(
            "RDAP error {code} for {domain}"
        )));
    }

    let name_servers: Vec<String> = rdap
        .nameservers
        .iter()
        .filter_map(|ns| ns.ldh_name.as_deref())
        .map(|s| s.to_lowercase())
        .collect();

    Ok(WhoisResult {
        domain: rdap
            .ldh_name
            .unwrap_or_else(|| domain.to_string())
            .to_lowercase(),
        registrar: extract_registrar_name(&rdap.entities),
        creation_date: find_event_date(&rdap.events, "registration"),
        expiration_date: find_event_date(&rdap.events, "expiration"),
        updated_date: find_event_date(&rdap.events, "last changed"),
        name_servers,
        status: rdap.status,
        raw: body,
        source: WhoisSource::Rdap,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_find_rdap_server_com() {
        let url = find_rdap_server("example.com");
        assert!(url.is_some(), "should find RDAP server for .com");
        assert!(
            url.unwrap().contains("verisign"),
            "expected VeriSign RDAP for .com"
        );
    }

    #[test]
    fn test_find_rdap_server_net() {
        let url = find_rdap_server("example.net");
        assert!(url.is_some(), "should find RDAP server for .net");
    }

    #[test]
    fn test_find_rdap_server_unknown_tld() {
        let url = find_rdap_server("example.zzzzzzz");
        assert!(url.is_none(), "unknown TLD should return None");
    }

    #[test]
    fn test_find_rdap_server_case_insensitive() {
        let url = find_rdap_server("EXAMPLE.COM");
        assert!(url.is_some());
    }

    #[test]
    fn test_find_rdap_server_trailing_dot() {
        let url = find_rdap_server("example.com.");
        assert!(url.is_some(), "trailing dot should not break bootstrap lookup");
    }

    #[test]
    fn test_find_event_date() {
        let events = vec![
            RdapEvent {
                event_action: "registration".to_string(),
                event_date: Some("2020-01-01T00:00:00Z".to_string()),
            },
            RdapEvent {
                event_action: "expiration".to_string(),
                event_date: Some("2025-01-01T00:00:00Z".to_string()),
            },
        ];
        assert_eq!(
            find_event_date(&events, "registration"),
            Some("2020-01-01T00:00:00Z".to_string())
        );
        assert_eq!(
            find_event_date(&events, "expiration"),
            Some("2025-01-01T00:00:00Z".to_string())
        );
        assert_eq!(find_event_date(&events, "transfer"), None);
    }

    #[test]
    fn test_extract_registrar_name_from_vcard() {
        let entity = RdapEntity {
            roles: vec!["registrar".to_string()],
            vcard_array: serde_json::json!([
                "vcard",
                [
                    ["version", {}, "text", "4.0"],
                    ["fn", {}, "text", "Example Registrar, Inc."]
                ]
            ]),
        };
        assert_eq!(
            extract_registrar_name(&[entity]),
            Some("Example Registrar, Inc.".to_string())
        );
    }

    #[test]
    fn test_extract_registrar_name_no_registrar_role() {
        let entity = RdapEntity {
            roles: vec!["technical".to_string()],
            vcard_array: serde_json::json!(["vcard", [["fn", {}, "text", "Tech Contact"]]]),
        };
        assert_eq!(extract_registrar_name(&[entity]), None);
    }

    #[test]
    fn test_extract_registrar_name_empty_entities() {
        assert_eq!(extract_registrar_name(&[]), None);
    }

    #[tokio::test]
    #[ignore = "requires network access"]
    async fn test_rdap_lookup_real() {
        let result = rdap_lookup("google.com").await;
        assert!(result.is_ok(), "RDAP lookup should succeed: {result:?}");
        let info = result.unwrap();
        assert!(
            info.domain.contains("google"),
            "domain should contain 'google'"
        );
        assert_eq!(info.source, WhoisSource::Rdap);
        assert!(info.registrar.is_some(), "registrar should be present");
        assert!(!info.name_servers.is_empty(), "should have name servers");
    }
}
