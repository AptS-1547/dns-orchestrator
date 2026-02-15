//! MCP Server implementation for DNS Orchestrator.
//!
//! Exposes 8 tools for AI agents to interact with DNS management functionality.

use async_trait::async_trait;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use std::sync::Arc;
use tokio::time::{Duration, timeout};

use dns_orchestrator_core::error::CoreError;
use dns_orchestrator_core::services::{
    AccountService, DnsService, DomainMetadataService, DomainService, ServiceContext,
};
use dns_orchestrator_core::types::DnsRecordType;
use dns_orchestrator_toolbox::{
    DnsLookupResult, DnsPropagationResult, DnsQueryType, DnssecResult, IpLookupResult,
    ToolboxError, ToolboxResult, ToolboxService, WhoisResult,
};

use crate::mcp_types::{
    McpAccount, McpAppDomain, McpDnsLookupResult, McpDnsPropagationResult, McpDnsRecord,
    McpDnssecResult, McpIpLookupResult, McpPaginatedResponse, McpWhoisResult,
};
use crate::schemas::{
    DnsLookupParams, DnsPropagationCheckParams, DnssecCheckParams, IpLookupParams,
    ListAccountsParams, ListDomainsParams, ListRecordsParams, WhoisLookupParams,
};

// Timeout constants for external service calls
const DNS_LOOKUP_TIMEOUT_SECS: u64 = 30;
const WHOIS_LOOKUP_TIMEOUT_SECS: u64 = 30;
const IP_LOOKUP_TIMEOUT_SECS: u64 = 15;
const DNS_PROPAGATION_TIMEOUT_SECS: u64 = 60;
const DNSSEC_CHECK_TIMEOUT_SECS: u64 = 30;

#[derive(Clone, Copy)]
struct ToolTimeouts {
    dns_lookup: Duration,
    whois_lookup: Duration,
    ip_lookup: Duration,
    dns_propagation_check: Duration,
    dnssec_check: Duration,
}

impl Default for ToolTimeouts {
    fn default() -> Self {
        Self {
            dns_lookup: Duration::from_secs(DNS_LOOKUP_TIMEOUT_SECS),
            whois_lookup: Duration::from_secs(WHOIS_LOOKUP_TIMEOUT_SECS),
            ip_lookup: Duration::from_secs(IP_LOOKUP_TIMEOUT_SECS),
            dns_propagation_check: Duration::from_secs(DNS_PROPAGATION_TIMEOUT_SECS),
            dnssec_check: Duration::from_secs(DNSSEC_CHECK_TIMEOUT_SECS),
        }
    }
}

#[async_trait]
trait ToolboxGateway: Send + Sync {
    async fn dns_lookup(
        &self,
        domain: &str,
        record_type: DnsQueryType,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnsLookupResult>;

    async fn whois_lookup(&self, domain: &str) -> ToolboxResult<WhoisResult>;

    async fn ip_lookup(&self, query: &str) -> ToolboxResult<IpLookupResult>;

    async fn dns_propagation_check(
        &self,
        domain: &str,
        record_type: DnsQueryType,
    ) -> ToolboxResult<DnsPropagationResult>;

    async fn dnssec_check(
        &self,
        domain: &str,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnssecResult>;
}

#[derive(Default)]
struct DefaultToolboxGateway;

#[async_trait]
impl ToolboxGateway for DefaultToolboxGateway {
    async fn dns_lookup(
        &self,
        domain: &str,
        record_type: DnsQueryType,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnsLookupResult> {
        ToolboxService::dns_lookup(domain, record_type, nameserver).await
    }

    async fn whois_lookup(&self, domain: &str) -> ToolboxResult<WhoisResult> {
        ToolboxService::whois_lookup(domain).await
    }

    async fn ip_lookup(&self, query: &str) -> ToolboxResult<IpLookupResult> {
        ToolboxService::ip_lookup(query).await
    }

    async fn dns_propagation_check(
        &self,
        domain: &str,
        record_type: DnsQueryType,
    ) -> ToolboxResult<DnsPropagationResult> {
        ToolboxService::dns_propagation_check(domain, record_type).await
    }

    async fn dnssec_check(
        &self,
        domain: &str,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnssecResult> {
        ToolboxService::dnssec_check(domain, nameserver).await
    }
}

/// Sanitize error messages to prevent sensitive information leakage.
///
/// Logs the full error to stderr but returns a generic message to the client.
fn sanitize_internal_error(error: impl std::fmt::Display, context: &str) -> McpError {
    log::error!("{context} error: {error}");
    McpError::internal_error(
        format!("{context} failed - check server logs for details"),
        None,
    )
}

/// Map a [`CoreError`] to either a user-facing tool error or a sanitized MCP error.
///
/// Expected errors (bad input, not found) are returned as `CallToolResult` with
/// `is_error = true` so the LLM can read the message and self-correct.
/// Unexpected errors (storage, network) are sanitized to prevent information leakage.
fn map_core_error(error: CoreError, context: &str) -> Result<CallToolResult, McpError> {
    if error.is_expected() {
        log::warn!("{context} expected error: {error}");
        Ok(CallToolResult::error(vec![Content::text(
            error.to_string(),
        )]))
    } else {
        Err(sanitize_internal_error(error, context))
    }
}

fn map_toolbox_error(context: &str, error: &ToolboxError) -> McpError {
    log::warn!("{context} error: {error}");
    McpError::internal_error(error.to_string(), None)
}

/// Execute a toolbox operation with timeout, error mapping, type conversion,
/// and compact JSON serialization.
async fn run_toolbox_tool<T, U: serde::Serialize + From<T>>(
    duration: Duration,
    future: impl std::future::Future<Output = ToolboxResult<T>>,
    tool_name: &str,
) -> Result<CallToolResult, McpError> {
    let result = timeout(duration, future)
        .await
        .map_err(|_| McpError::internal_error(format!("{tool_name} timeout"), None))?
        .map_err(|e| map_toolbox_error(tool_name, &e))?;

    let mapped: U = result.into();
    let json = serde_json::to_string(&mapped)
        .map_err(|e| sanitize_internal_error(e, &format!("Serialize {tool_name} result")))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// MCP Server for DNS Orchestrator.
///
/// Provides AI agents with access to DNS management functionality
/// through the Model Context Protocol.
#[derive(Clone)]
pub struct DnsOrchestratorMcp {
    /// Account service for account operations.
    account_service: Arc<AccountService>,
    /// Domain service for domain operations.
    domain_service: Arc<DomainService>,
    /// DNS service for record operations.
    dns_service: Arc<DnsService>,
    /// Toolbox gateway for network utilities.
    toolbox: Arc<dyn ToolboxGateway>,
    /// Timeout configuration for toolbox calls.
    timeouts: ToolTimeouts,
    /// Tool router generated by macro.
    tool_router: ToolRouter<Self>,
}

impl DnsOrchestratorMcp {
    /// Create a new MCP server instance.
    #[must_use]
    pub fn new(
        ctx: &Arc<ServiceContext>,
        account_service: Arc<AccountService>,
        domain_metadata_service: Arc<DomainMetadataService>,
    ) -> Self {
        Self::with_toolbox_and_timeouts(
            ctx,
            account_service,
            domain_metadata_service,
            Arc::new(DefaultToolboxGateway),
            ToolTimeouts::default(),
        )
    }

    fn with_toolbox_and_timeouts(
        ctx: &Arc<ServiceContext>,
        account_service: Arc<AccountService>,
        domain_metadata_service: Arc<DomainMetadataService>,
        toolbox: Arc<dyn ToolboxGateway>,
        timeouts: ToolTimeouts,
    ) -> Self {
        let domain_service = Arc::new(DomainService::new(Arc::clone(ctx), domain_metadata_service));
        let dns_service = Arc::new(DnsService::new(Arc::clone(ctx)));

        Self {
            account_service,
            domain_service,
            dns_service,
            toolbox,
            timeouts,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl DnsOrchestratorMcp {
    /// List all configured DNS accounts.
    #[tool(
        description = "List all configured DNS provider accounts (Cloudflare, Aliyun, DNSPod, Huaweicloud)"
    )]
    async fn list_accounts(
        &self,
        _params: Parameters<ListAccountsParams>,
    ) -> Result<CallToolResult, McpError> {
        let accounts = match self.account_service.list_accounts().await {
            Ok(v) => v,
            Err(e) => return map_core_error(e, "List accounts"),
        };

        let compact: Vec<McpAccount> = accounts.into_iter().map(Into::into).collect();
        let json = serde_json::to_string(&compact)
            .map_err(|e| sanitize_internal_error(e, "Serialize accounts"))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List domains for a specific account.
    #[tool(description = "List all DNS domains for a specific account with pagination support")]
    async fn list_domains(
        &self,
        Parameters(params): Parameters<ListDomainsParams>,
    ) -> Result<CallToolResult, McpError> {
        // Limit page_size to prevent resource exhaustion
        let page_size = params.page_size.map(|s| s.min(100));

        let result = match self
            .domain_service
            .list_domains(&params.account_id, params.page, page_size)
            .await
        {
            Ok(v) => v,
            Err(e) => return map_core_error(e, "List domains"),
        };

        let compact: McpPaginatedResponse<McpAppDomain> = result.into();
        let json = serde_json::to_string(&compact)
            .map_err(|e| sanitize_internal_error(e, "Serialize domains"))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// List DNS records for a specific domain.
    #[tool(
        description = "List DNS records for a specific domain with pagination and filtering support"
    )]
    async fn list_records(
        &self,
        Parameters(params): Parameters<ListRecordsParams>,
    ) -> Result<CallToolResult, McpError> {
        let record_type = params
            .record_type
            .and_then(|t| match t.to_uppercase().as_str() {
                "A" => Some(DnsRecordType::A),
                "AAAA" => Some(DnsRecordType::Aaaa),
                "CNAME" => Some(DnsRecordType::Cname),
                "MX" => Some(DnsRecordType::Mx),
                "TXT" => Some(DnsRecordType::Txt),
                "NS" => Some(DnsRecordType::Ns),
                "SRV" => Some(DnsRecordType::Srv),
                "CAA" => Some(DnsRecordType::Caa),
                _ => None,
            });

        // Limit page_size to prevent resource exhaustion
        let page_size = params.page_size.map(|s| s.min(100));

        let result = match self
            .dns_service
            .list_records(
                &params.account_id,
                &params.domain_id,
                params.page,
                page_size,
                params.keyword,
                record_type,
            )
            .await
        {
            Ok(v) => v,
            Err(e) => return map_core_error(e, "List records"),
        };

        let compact: McpPaginatedResponse<McpDnsRecord> = result.into();
        let json = serde_json::to_string(&compact)
            .map_err(|e| sanitize_internal_error(e, "Serialize records"))?;

        Ok(CallToolResult::success(vec![Content::text(json)]))
    }

    /// Perform DNS lookup.
    #[tool(
        description = "Perform DNS lookup for a domain (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL)"
    )]
    async fn dns_lookup(
        &self,
        Parameters(params): Parameters<DnsLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        run_toolbox_tool::<DnsLookupResult, McpDnsLookupResult>(
            self.timeouts.dns_lookup,
            self.toolbox.dns_lookup(
                &params.domain,
                params.record_type,
                params.nameserver.as_deref(),
            ),
            "DNS lookup",
        )
        .await
    }

    /// Perform WHOIS lookup.
    #[tool(
        description = "Query WHOIS information for a domain (registrar, registration dates, name servers)"
    )]
    async fn whois_lookup(
        &self,
        Parameters(params): Parameters<WhoisLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        run_toolbox_tool::<WhoisResult, McpWhoisResult>(
            self.timeouts.whois_lookup,
            self.toolbox.whois_lookup(&params.domain),
            "WHOIS lookup",
        )
        .await
    }

    /// Perform IP geolocation lookup.
    #[tool(
        description = "Look up geolocation data for an IP address or domain (country, region, city, ISP, ASN)"
    )]
    async fn ip_lookup(
        &self,
        Parameters(params): Parameters<IpLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        run_toolbox_tool::<IpLookupResult, McpIpLookupResult>(
            self.timeouts.ip_lookup,
            self.toolbox.ip_lookup(&params.query),
            "IP lookup",
        )
        .await
    }

    /// Check DNS propagation.
    #[tool(description = "Check DNS record propagation across 13 global DNS servers")]
    async fn dns_propagation_check(
        &self,
        Parameters(params): Parameters<DnsPropagationCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        run_toolbox_tool::<DnsPropagationResult, McpDnsPropagationResult>(
            self.timeouts.dns_propagation_check,
            self.toolbox
                .dns_propagation_check(&params.domain, params.record_type),
            "DNS propagation check",
        )
        .await
    }

    /// Check DNSSEC status.
    #[tool(description = "Validate DNSSEC deployment for a domain (DNSKEY, DS, RRSIG records)")]
    async fn dnssec_check(
        &self,
        Parameters(params): Parameters<DnssecCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        run_toolbox_tool::<DnssecResult, McpDnssecResult>(
            self.timeouts.dnssec_check,
            self.toolbox
                .dnssec_check(&params.domain, params.nameserver.as_deref()),
            "DNSSEC check",
        )
        .await
    }
}

#[tool_handler]
impl ServerHandler for DnsOrchestratorMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::LATEST,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "DNS Orchestrator MCP Server - Access DNS accounts and records across multiple providers \
                 (Cloudflare, Aliyun, DNSPod, Huaweicloud). \
                 Use list_accounts to see available accounts, list_domains to see domains, \
                 and list_records to view DNS records. \
                 Network diagnostic tools (dns_lookup, whois_lookup, ip_lookup, dns_propagation_check, dnssec_check) \
                 work without any account configuration."
                    .into(),
            ),
        }
    }
}

#[cfg(test)]
#[path = "test_mocks.rs"]
#[allow(clippy::unwrap_used, clippy::panic)]
pub(crate) mod test_mocks;

#[cfg(test)]
#[path = "server_tests.rs"]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests;

#[cfg(test)]
#[path = "client_integration_tests.rs"]
#[allow(clippy::unwrap_used, clippy::panic)]
mod client_integration_tests;
