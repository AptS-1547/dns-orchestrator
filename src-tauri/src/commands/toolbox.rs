use tauri::State;

use crate::types::{
    ApiResponse, CertChainItem, DnsLookupRecord, DnsLookupResult, IpGeoInfo, IpLookupResult,
    SslCertInfo, SslCheckResult, WhoisResult,
};
use crate::AppState;

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

/// WHOIS 查询
#[tauri::command]
pub async fn whois_lookup(
    state: State<'_, AppState>,
    domain: String,
) -> Result<ApiResponse<WhoisResult>, String> {
    let result = state
        .toolbox_service
        .whois_lookup(&domain)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_whois_result(result)))
}

/// DNS 查询
#[tauri::command]
pub async fn dns_lookup(
    state: State<'_, AppState>,
    domain: String,
    record_type: String,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnsLookupResult>, String> {
    let result = state
        .toolbox_service
        .dns_lookup(&domain, &record_type, nameserver.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_dns_lookup_result(result)))
}

/// IP/域名 地理位置查询
#[tauri::command]
pub async fn ip_lookup(
    state: State<'_, AppState>,
    query: String,
) -> Result<ApiResponse<IpLookupResult>, String> {
    let result = state
        .toolbox_service
        .ip_lookup(&query)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_ip_lookup_result(result)))
}

/// SSL 证书检查
#[tauri::command]
pub async fn ssl_check(
    state: State<'_, AppState>,
    domain: String,
    port: Option<u16>,
) -> Result<ApiResponse<SslCheckResult>, String> {
    let result = state
        .toolbox_service
        .ssl_check(&domain, port)
        .await
        .map_err(|e| e.to_string())?;

    Ok(ApiResponse::success(convert_ssl_check_result(result)))
}
