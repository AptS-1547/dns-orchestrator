//! MCP tool parameter schemas
//!
//! Defines the input parameter structures for all MCP tools.
//! All structs derive `Debug`, `Deserialize`, and `JsonSchema` as required by rmcp.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for `list_accounts` tool.
///
/// This tool takes no parameters, but we need an empty struct for the schema.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListAccountsParams {}

/// Parameters for `list_domains` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListDomainsParams {
    /// The account ID to list domains for.
    #[schemars(description = "The account ID to list domains for")]
    pub account_id: String,

    /// Page number (1-indexed, default: 1).
    #[schemars(description = "Page number (1-indexed, default: 1)")]
    pub page: Option<u32>,

    /// Number of items per page (default: 20).
    #[schemars(description = "Number of items per page (default: 20)")]
    pub page_size: Option<u32>,
}

/// Parameters for `list_records` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRecordsParams {
    /// The account ID.
    #[schemars(description = "The account ID")]
    pub account_id: String,

    /// The domain ID to list records for.
    #[schemars(description = "The domain ID to list records for")]
    pub domain_id: String,

    /// Page number (1-indexed, default: 1).
    #[schemars(description = "Page number (1-indexed, default: 1)")]
    pub page: Option<u32>,

    /// Number of items per page (default: 20).
    #[schemars(description = "Number of items per page (default: 20)")]
    pub page_size: Option<u32>,

    /// Keyword to filter records by name.
    #[schemars(description = "Keyword to filter records by name")]
    pub keyword: Option<String>,

    /// Record type filter (e.g., A, AAAA, CNAME, MX, TXT).
    #[schemars(description = "Record type filter (e.g., A, AAAA, CNAME, MX, TXT)")]
    pub record_type: Option<String>,
}

/// Parameters for `dns_lookup` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DnsLookupParams {
    /// The domain name to query.
    #[schemars(description = "The domain name to query")]
    pub domain: String,

    /// DNS record type (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL).
    #[schemars(
        description = "DNS record type (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL)"
    )]
    pub record_type: String,

    /// Optional custom nameserver IP address.
    #[schemars(description = "Optional custom nameserver IP address")]
    pub nameserver: Option<String>,
}

/// Parameters for `whois_lookup` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WhoisLookupParams {
    /// The domain name to query WHOIS information for.
    #[schemars(description = "The domain name to query WHOIS information for")]
    pub domain: String,
}

/// Parameters for `ip_lookup` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IpLookupParams {
    /// IP address or domain name to look up geolocation for.
    #[schemars(description = "IP address or domain name to look up geolocation for")]
    pub query: String,
}

/// Parameters for `dns_propagation_check` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DnsPropagationCheckParams {
    /// The domain name to check.
    #[schemars(description = "The domain name to check")]
    pub domain: String,

    /// DNS record type to check (e.g., A, AAAA, CNAME).
    #[schemars(description = "DNS record type to check (e.g., A, AAAA, CNAME)")]
    pub record_type: String,
}

/// Parameters for `dnssec_check` tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DnssecCheckParams {
    /// The domain name to check DNSSEC for.
    #[schemars(description = "The domain name to check DNSSEC for")]
    pub domain: String,

    /// Optional custom nameserver IP address.
    #[schemars(description = "Optional custom nameserver IP address")]
    pub nameserver: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    use schemars::schema_for;

    #[test]
    fn list_domains_deserializes_required_and_optional_fields() {
        let json = serde_json::json!({
            "account_id": "acc-1",
            "page": 2,
            "page_size": 50
        });

        let params: ListDomainsParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.account_id, "acc-1");
        assert_eq!(params.page, Some(2));
        assert_eq!(params.page_size, Some(50));
    }

    #[test]
    fn list_domains_missing_account_id_fails() {
        let json = serde_json::json!({ "page": 1, "page_size": 20 });
        let result: serde_json::Result<ListDomainsParams> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn list_records_missing_required_fields_fails() {
        let json = serde_json::json!({
            "page": 1,
            "page_size": 20,
            "keyword": "www"
        });
        let result: serde_json::Result<ListRecordsParams> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn dns_lookup_nameserver_is_optional() {
        let without_nameserver = serde_json::json!({
            "domain": "example.com",
            "record_type": "A"
        });
        let with_nameserver = serde_json::json!({
            "domain": "example.com",
            "record_type": "A",
            "nameserver": "8.8.8.8"
        });

        let parsed_without: DnsLookupParams = serde_json::from_value(without_nameserver).unwrap();
        let parsed_with: DnsLookupParams = serde_json::from_value(with_nameserver).unwrap();

        assert!(parsed_without.nameserver.is_none());
        assert_eq!(parsed_with.nameserver, Some("8.8.8.8".to_string()));
    }

    #[test]
    fn schema_marks_required_fields_for_list_records() {
        let schema = schema_for!(ListRecordsParams);
        let json = serde_json::to_value(&schema).unwrap();
        let required = json
            .get("required")
            .and_then(serde_json::Value::as_array)
            .unwrap();

        assert!(required.iter().any(|v| v == "account_id"));
        assert!(required.iter().any(|v| v == "domain_id"));
        assert!(!required.iter().any(|v| v == "page"));
        assert!(!required.iter().any(|v| v == "record_type"));
    }

    #[test]
    fn list_accounts_accepts_empty_object() {
        let params: ListAccountsParams = serde_json::from_value(serde_json::json!({})).unwrap();
        let _ = params;
    }
}
