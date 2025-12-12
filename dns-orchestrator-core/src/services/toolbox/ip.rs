//! IP 地理位置查询模块

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use serde::Deserialize;

use crate::error::{CoreError, CoreResult};
use crate::types::{IpGeoInfo, IpLookupResult};

/// ipwhois.io 响应结构
#[derive(Deserialize)]
struct IpWhoisResponse {
    ip: String,
    success: bool,
    message: Option<String>,
    #[serde(rename = "type")]
    ip_type: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
    region: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    timezone: Option<IpWhoisTimezone>,
    connection: Option<IpWhoisConnection>,
}

#[derive(Deserialize)]
struct IpWhoisTimezone {
    id: Option<String>,
}

#[derive(Deserialize)]
struct IpWhoisConnection {
    asn: Option<i64>,
    org: Option<String>,
    isp: Option<String>,
}

/// 查询单个 IP 的地理位置
async fn lookup_single_ip(ip: &str, client: &reqwest::Client) -> CoreResult<IpGeoInfo> {
    let url = format!(
        "https://ipwho.is/{ip}?fields=ip,success,message,type,country,country_code,region,city,latitude,longitude,timezone,connection"
    );

    let response: IpWhoisResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| CoreError::NetworkError(format!("请求失败: {e}")))?
        .json()
        .await
        .map_err(|e| CoreError::NetworkError(format!("解析失败: {e}")))?;

    if !response.success {
        let error_msg = match response.message.as_deref() {
            Some("You've hit the monthly limit") => {
                "IP 查询服务已达本月限额，请稍后再试".to_string()
            }
            Some("Invalid IP address") => "无效的 IP 地址".to_string(),
            Some("Reserved range") => "该 IP 属于保留地址段，无法查询".to_string(),
            Some(msg) => format!("查询失败: {msg}"),
            None => "查询失败".to_string(),
        };
        return Err(CoreError::NetworkError(error_msg));
    }

    let ip_version = response.ip_type.unwrap_or_else(|| {
        if response.ip.contains(':') {
            "IPv6"
        } else {
            "IPv4"
        }
        .to_string()
    });

    let (isp, org, asn) = response.connection.map_or((None, None, None), |conn| {
        (
            conn.isp,
            conn.org.clone(),
            conn.asn.map(|n| format!("AS{n}")),
        )
    });

    let timezone = response.timezone.and_then(|tz| tz.id);

    Ok(IpGeoInfo {
        ip: response.ip,
        ip_version,
        country: response.country,
        country_code: response.country_code,
        region: response.region,
        city: response.city,
        latitude: response.latitude,
        longitude: response.longitude,
        timezone,
        isp,
        org: org.clone(),
        asn,
        as_name: org,
    })
}

/// IP/域名 地理位置查询
pub async fn ip_lookup(query: &str) -> CoreResult<IpLookupResult> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err(CoreError::ValidationError(
            "请输入 IP 地址或域名".to_string(),
        ));
    }

    let client = reqwest::Client::new();

    // 检查是否为 IP 地址
    if query.parse::<std::net::IpAddr>().is_ok() {
        let result = lookup_single_ip(&query, &client).await?;
        return Ok(IpLookupResult {
            query,
            is_domain: false,
            results: vec![result],
        });
    }

    // 作为域名处理，解析 A 和 AAAA 记录
    let provider = TokioConnectionProvider::default();
    let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
        .with_options(ResolverOpts::default())
        .build();

    let mut ips: Vec<String> = Vec::new();

    // 解析 IPv4 (A 记录)
    if let Ok(response) = resolver.ipv4_lookup(&query).await {
        for ip in response.iter() {
            ips.push(ip.to_string());
        }
    }

    // 解析 IPv6 (AAAA 记录)
    if let Ok(response) = resolver.ipv6_lookup(&query).await {
        for ip in response.iter() {
            ips.push(ip.to_string());
        }
    }

    if ips.is_empty() {
        return Err(CoreError::NetworkError(format!("无法解析域名: {query}")));
    }

    // 查询每个 IP 的地理位置
    let mut results = Vec::new();
    for ip in ips {
        match lookup_single_ip(&ip, &client).await {
            Ok(info) => results.push(info),
            Err(e) => {
                log::warn!("查询 IP {ip} 失败: {e}");
            }
        }
    }

    if results.is_empty() {
        return Err(CoreError::NetworkError(
            "所有 IP 地址查询均失败".to_string(),
        ));
    }

    Ok(IpLookupResult {
        query,
        is_domain: true,
        results,
    })
}
