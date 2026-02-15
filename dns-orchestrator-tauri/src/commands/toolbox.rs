use dns_orchestrator_toolbox::{
    DnsLookupResult, DnsPropagationResult, DnsQueryType, DnssecResult, HttpHeaderCheckRequest,
    HttpHeaderCheckResult, IpLookupResult, SslCheckResult, ToolboxService, WhoisResult,
};

use crate::error::AppError;
use crate::types::ApiResponse;

/// WHOIS 查询
#[tauri::command]
pub async fn whois_lookup(domain: String) -> Result<ApiResponse<WhoisResult>, AppError> {
    let result = ToolboxService::whois_lookup(&domain).await?;
    Ok(ApiResponse::success(result))
}

/// DNS 查询
#[tauri::command]
pub async fn dns_lookup(
    domain: String,
    record_type: DnsQueryType,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnsLookupResult>, AppError> {
    let result = ToolboxService::dns_lookup(&domain, record_type, nameserver.as_deref()).await?;
    Ok(ApiResponse::success(result))
}

/// IP/域名 地理位置查询
#[tauri::command]
pub async fn ip_lookup(query: String) -> Result<ApiResponse<IpLookupResult>, AppError> {
    let result = ToolboxService::ip_lookup(&query).await?;
    Ok(ApiResponse::success(result))
}

/// SSL 证书检查
#[tauri::command]
pub async fn ssl_check(
    domain: String,
    port: Option<u16>,
) -> Result<ApiResponse<SslCheckResult>, AppError> {
    let result = ToolboxService::ssl_check(&domain, port).await?;
    Ok(ApiResponse::success(result))
}

/// HTTP 头检查
#[tauri::command]
pub async fn http_header_check(
    request: HttpHeaderCheckRequest,
) -> Result<ApiResponse<HttpHeaderCheckResult>, AppError> {
    let result = ToolboxService::http_header_check(&request).await?;
    Ok(ApiResponse::success(result))
}

/// DNS 传播检查
#[tauri::command]
pub async fn dns_propagation_check(
    domain: String,
    record_type: DnsQueryType,
) -> Result<ApiResponse<DnsPropagationResult>, AppError> {
    let result = ToolboxService::dns_propagation_check(&domain, record_type).await?;
    Ok(ApiResponse::success(result))
}

/// DNSSEC 验证
#[tauri::command]
pub async fn dnssec_check(
    domain: String,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnssecResult>, AppError> {
    let result = ToolboxService::dnssec_check(&domain, nameserver.as_deref()).await?;
    Ok(ApiResponse::success(result))
}
