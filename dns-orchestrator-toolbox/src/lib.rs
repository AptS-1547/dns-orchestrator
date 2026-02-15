//! Async network diagnostic toolkit for DNS and domain analysis.
//!
//! Provides stateless, independent utility functions for common network diagnostics:
//!
//! - **WHOIS lookup** -- query domain registration info with structured field parsing
//! - **DNS lookup** -- resolve any record type (A, AAAA, MX, TXT, NS, CNAME, SOA, SRV, CAA, PTR)
//!   with optional custom nameserver
//! - **DNS propagation check** -- test record consistency across 13 global DNS servers
//! - **DNSSEC validation** -- verify DNSSEC deployment and validation status
//! - **IP geolocation** -- look up country, region, city, ISP, ASN for IPs or domains
//! - **SSL certificate check** -- inspect certificate chain, validity, SAN, expiration via rustls
//! - **HTTP header analysis** -- send requests with any method/headers and get security header
//!   recommendations
//!
//! All functions are stateless -- no shared state, no business logic dependencies.
//! Every public method lives on [`ToolboxService`] as an async associated function.
//!
//! # Notes
//!
//! - `ToolboxService::dns_lookup` uses the host system DNS configuration by default; pass a
//!   nameserver IP to query a specific resolver.
//! - `ToolboxService::ip_lookup` uses the public ipwho.is API for geolocation and may be
//!   rate-limited.
//!
//! # Quick start
//!
//! ```toml
//! [dependencies]
//! dns-orchestrator-toolbox = "0.1"
//! ```
//!
//! ```rust,no_run
//! use dns_orchestrator_toolbox::{ToolboxService, DnsQueryType};
//!
//! # async fn example() -> dns_orchestrator_toolbox::ToolboxResult<()> {
//! // WHOIS
//! let whois = ToolboxService::whois_lookup("example.com").await?;
//!
//! // DNS lookup with system default nameserver
//! let dns = ToolboxService::dns_lookup("example.com", DnsQueryType::A, None).await?;
//!
//! // DNS propagation across global servers
//! let prop = ToolboxService::dns_propagation_check("example.com", DnsQueryType::A).await?;
//! println!("Consistency: {:.1}%", prop.consistency_percentage);
//!
//! // IP geolocation (accepts IP or domain)
//! let ip = ToolboxService::ip_lookup("1.1.1.1").await?;
//!
//! // SSL certificate inspection
//! let ssl = ToolboxService::ssl_check("example.com", None).await?;
//! println!("Connection: {}", ssl.connection_status);
//!
//! // DNSSEC validation
//! let dnssec = ToolboxService::dnssec_check("example.com", None).await?;
//! println!("DNSSEC status: {}", dnssec.validation_status);
//! # Ok(())
//! # }
//! ```

mod error;
mod services;
mod types;

pub use error::{ToolboxError, ToolboxResult};
pub use services::ToolboxService;
pub use types::{
    CertChainItem, ConnectionStatus, DnsLookupRecord, DnsLookupResult, DnsPropagationResult,
    DnsPropagationServer, DnsPropagationServerResult, DnsQueryType, DnskeyRecord, DnssecResult,
    DnssecValidationStatus, DsRecord, HttpHeader, HttpHeaderCheckRequest, HttpHeaderCheckResult,
    HttpMethod, IpGeoInfo, IpLookupResult, PropagationStatus, RrsigRecord, SecurityHeaderAnalysis,
    SecurityHeaderStatus, SslCertInfo, SslCheckResult, WhoisResult,
};
