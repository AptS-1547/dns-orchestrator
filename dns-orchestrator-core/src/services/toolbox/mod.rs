//! 工具箱服务模块

mod dns;
mod ip;
mod ssl;
mod whois;

use crate::error::CoreResult;
use crate::types::{DnsLookupResult, IpLookupResult, WhoisResult};

/// 嵌入 WHOIS 服务器配置
const WHOIS_SERVERS: &str = include_str!("whois_servers.json");

/// 工具箱服务
pub struct ToolboxService;

impl ToolboxService {
    /// 创建工具箱服务实例
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// WHOIS 查询
    pub async fn whois_lookup(&self, domain: &str) -> CoreResult<WhoisResult> {
        whois::whois_lookup(domain, WHOIS_SERVERS).await
    }

    /// DNS 查询
    pub async fn dns_lookup(
        &self,
        domain: &str,
        record_type: &str,
        nameserver: Option<&str>,
    ) -> CoreResult<DnsLookupResult> {
        dns::dns_lookup(domain, record_type, nameserver).await
    }

    /// IP/域名 地理位置查询
    pub async fn ip_lookup(&self, query: &str) -> CoreResult<IpLookupResult> {
        ip::ip_lookup(query).await
    }

    /// SSL 证书检查
    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    pub async fn ssl_check(
        &self,
        domain: &str,
        port: Option<u16>,
    ) -> CoreResult<crate::types::SslCheckResult> {
        ssl::ssl_check(domain, port).await
    }
}

impl Default for ToolboxService {
    fn default() -> Self {
        Self::new()
    }
}
