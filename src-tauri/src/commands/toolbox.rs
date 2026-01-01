use dns_orchestrator_core::services::ToolboxService;

use crate::types::{
    ApiResponse, CertChainItem, DnsLookupRecord, DnsLookupResult, HttpHeader,
    HttpHeaderCheckRequest, HttpHeaderCheckResult, HttpMethod, IpGeoInfo, IpLookupResult,
    SecurityHeaderAnalysis, SslCertInfo, SslCheckResult, WhoisResult,
};

// 类型转换辅助函数
fn convert_whois_result(result: dns_orchestrator_core::types::WhoisResult) -> WhoisResult {
    WhoisResult {
        domain: result.domain,
        registrar: result.registrar,
        creation_date: result.creation_date,
        expiration_date: result.expiration_date,
        updated_date: result.updated_date,
        name_servers: result.name_servers,
        status: result.status,
        raw: result.raw,
    }
}

fn convert_dns_lookup_result(
    result: dns_orchestrator_core::types::DnsLookupResult,
) -> DnsLookupResult {
    DnsLookupResult {
        nameserver: result.nameserver,
        records: result
            .records
            .into_iter()
            .map(|r| DnsLookupRecord {
                record_type: r.record_type,
                name: r.name,
                value: r.value,
                ttl: r.ttl,
                priority: r.priority,
            })
            .collect(),
    }
}

fn convert_ip_lookup_result(
    result: dns_orchestrator_core::types::IpLookupResult,
) -> IpLookupResult {
    IpLookupResult {
        query: result.query,
        is_domain: result.is_domain,
        results: result
            .results
            .into_iter()
            .map(|r| IpGeoInfo {
                ip: r.ip,
                ip_version: r.ip_version,
                country: r.country,
                country_code: r.country_code,
                region: r.region,
                city: r.city,
                latitude: r.latitude,
                longitude: r.longitude,
                timezone: r.timezone,
                isp: r.isp,
                org: r.org,
                asn: r.asn,
                as_name: r.as_name,
            })
            .collect(),
    }
}

fn convert_ssl_check_result(
    result: dns_orchestrator_core::types::SslCheckResult,
) -> SslCheckResult {
    SslCheckResult {
        domain: result.domain,
        port: result.port,
        connection_status: result.connection_status,
        cert_info: result.cert_info.map(|info| SslCertInfo {
            domain: info.domain,
            issuer: info.issuer,
            subject: info.subject,
            valid_from: info.valid_from,
            valid_to: info.valid_to,
            days_remaining: info.days_remaining,
            is_expired: info.is_expired,
            is_valid: info.is_valid,
            san: info.san,
            serial_number: info.serial_number,
            signature_algorithm: info.signature_algorithm,
            certificate_chain: info
                .certificate_chain
                .into_iter()
                .map(|c| CertChainItem {
                    subject: c.subject,
                    issuer: c.issuer,
                    is_ca: c.is_ca,
                })
                .collect(),
        }),
        error: result.error,
    }
}

fn convert_http_method(method: HttpMethod) -> dns_orchestrator_core::types::HttpMethod {
    match method {
        HttpMethod::GET => dns_orchestrator_core::types::HttpMethod::GET,
        HttpMethod::HEAD => dns_orchestrator_core::types::HttpMethod::HEAD,
        HttpMethod::POST => dns_orchestrator_core::types::HttpMethod::POST,
        HttpMethod::PUT => dns_orchestrator_core::types::HttpMethod::PUT,
        HttpMethod::DELETE => dns_orchestrator_core::types::HttpMethod::DELETE,
        HttpMethod::PATCH => dns_orchestrator_core::types::HttpMethod::PATCH,
        HttpMethod::OPTIONS => dns_orchestrator_core::types::HttpMethod::OPTIONS,
    }
}

fn convert_http_header_check_request(
    request: HttpHeaderCheckRequest,
) -> dns_orchestrator_core::types::HttpHeaderCheckRequest {
    dns_orchestrator_core::types::HttpHeaderCheckRequest {
        url: request.url,
        method: convert_http_method(request.method),
        custom_headers: request
            .custom_headers
            .into_iter()
            .map(|h| dns_orchestrator_core::types::HttpHeader {
                name: h.name,
                value: h.value,
            })
            .collect(),
        body: request.body,
        content_type: request.content_type,
    }
}

fn convert_http_header_check_result(
    result: dns_orchestrator_core::types::HttpHeaderCheckResult,
) -> HttpHeaderCheckResult {
    HttpHeaderCheckResult {
        url: result.url,
        status_code: result.status_code,
        status_text: result.status_text,
        response_time_ms: result.response_time_ms,
        headers: result
            .headers
            .into_iter()
            .map(|h| HttpHeader {
                name: h.name,
                value: h.value,
            })
            .collect(),
        security_analysis: result
            .security_analysis
            .into_iter()
            .map(|s| SecurityHeaderAnalysis {
                name: s.name,
                present: s.present,
                value: s.value,
                status: s.status,
                recommendation: s.recommendation,
            })
            .collect(),
        content_length: result.content_length,
        raw_request: result.raw_request,
        raw_response: result.raw_response,
    }
}

/// WHOIS 查询
#[tauri::command]
pub async fn whois_lookup(domain: String) -> Result<ApiResponse<WhoisResult>, String> {
    let result = ToolboxService::whois_lookup(&domain)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_whois_result(result)))
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

    Ok(ApiResponse::success(convert_dns_lookup_result(result)))
}

/// IP/域名 地理位置查询
#[tauri::command]
pub async fn ip_lookup(query: String) -> Result<ApiResponse<IpLookupResult>, String> {
    let result = ToolboxService::ip_lookup(&query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_ip_lookup_result(result)))
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

    Ok(ApiResponse::success(convert_ssl_check_result(result)))
}

/// HTTP 头检查
#[tauri::command]
pub async fn http_header_check(
    request: HttpHeaderCheckRequest,
) -> Result<ApiResponse<HttpHeaderCheckResult>, String> {
    let core_request = convert_http_header_check_request(request);
    let result = ToolboxService::http_header_check(&core_request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_http_header_check_result(
        result,
    )))
}
