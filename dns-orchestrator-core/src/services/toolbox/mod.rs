//! 工具箱服务模块
//!
//! 提供各种 DNS 相关的工具函数，所有方法都是无状态的关联函数。

mod dns;
mod dns_propagation;
mod dnssec;
mod http_headers;
mod ip;
mod ssl;
mod whois;

use crate::error::CoreResult;
use crate::types::{
    DnsLookupResult, DnsPropagationResult, DnssecResult, HttpHeaderCheckResult, IpLookupResult,
    WhoisResult,
};

/// 嵌入 WHOIS 服务器配置
const WHOIS_SERVERS: &str = include_str!("whois_servers.json");

/// 工具箱服务（无状态，所有方法为关联函数）
pub struct ToolboxService;

impl ToolboxService {
    /// WHOIS 查询
    pub async fn whois_lookup(domain: &str) -> CoreResult<WhoisResult> {
        whois::whois_lookup(domain, WHOIS_SERVERS).await
    }

    /// DNS 查询
    pub async fn dns_lookup(
        domain: &str,
        record_type: &str,
        nameserver: Option<&str>,
    ) -> CoreResult<DnsLookupResult> {
        dns::dns_lookup(domain, record_type, nameserver).await
    }

    /// IP/域名 地理位置查询
    pub async fn ip_lookup(query: &str) -> CoreResult<IpLookupResult> {
        ip::ip_lookup(query).await
    }

    /// SSL 证书检查
    #[cfg(feature = "rustls")]
    pub async fn ssl_check(
        domain: &str,
        port: Option<u16>,
    ) -> CoreResult<crate::types::SslCheckResult> {
        ssl::ssl_check(domain, port).await
    }

    /// HTTP 头检查
    pub async fn http_header_check(
        request: &crate::types::HttpHeaderCheckRequest,
    ) -> CoreResult<HttpHeaderCheckResult> {
        http_headers::http_header_check(request).await
    }

    /// DNS 传播检查
    pub async fn dns_propagation_check(
        domain: &str,
        record_type: &str,
    ) -> CoreResult<DnsPropagationResult> {
        dns_propagation::dns_propagation_check(domain, record_type).await
    }

    /// DNSSEC 验证
    pub async fn dnssec_check(domain: &str, nameserver: Option<&str>) -> CoreResult<DnssecResult> {
        dnssec::dnssec_check(domain, nameserver).await
    }
}
