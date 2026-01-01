use dns_orchestrator_core::services::ToolboxService;
use dns_orchestrator_core::types::{
    DnsLookupResult, DnsPropagationResult, DnssecResult, HttpHeaderCheckRequest,
    HttpHeaderCheckResult, IpLookupResult, SslCheckResult, WhoisResult,
};

use crate::types::ApiResponse;

/// WHOIS 查询
#[tauri::command]
pub async fn whois_lookup(domain: String) -> Result<ApiResponse<WhoisResult>, String> {
    let result = ToolboxService::whois_lookup(&domain)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// DNS 查询
#[tauri::command]
pub async fn dns_lookup(
    domain: String,
    record_type: String,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnsLookupResult>, String> {
    let result = ToolboxService::dns_lookup(&domain, &record_type, nameserver.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// IP/域名 地理位置查询
#[tauri::command]
pub async fn ip_lookup(query: String) -> Result<ApiResponse<IpLookupResult>, String> {
    let result = ToolboxService::ip_lookup(&query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// SSL 证书检查
#[tauri::command]
pub async fn ssl_check(
    domain: String,
    port: Option<u16>,
) -> Result<ApiResponse<SslCheckResult>, String> {
    let result = ToolboxService::ssl_check(&domain, port)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// HTTP 头检查
#[tauri::command]
pub async fn http_header_check(
    request: HttpHeaderCheckRequest,
) -> Result<ApiResponse<HttpHeaderCheckResult>, String> {
    let result = ToolboxService::http_header_check(&request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// DNS 传播检查
#[tauri::command]
pub async fn dns_propagation_check(
    domain: String,
    record_type: String,
) -> Result<ApiResponse<DnsPropagationResult>, String> {
    let result = ToolboxService::dns_propagation_check(&domain, &record_type)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}

/// DNSSEC 验证
#[tauri::command]
pub async fn dnssec_check(
    domain: String,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnssecResult>, String> {
    let result = ToolboxService::dnssec_check(&domain, nameserver.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(result))
}
