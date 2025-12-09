use futures::future::join_all;
use hickory_resolver::{
    config::{NameServerConfigGroup, ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use regex::Regex;
use std::net::IpAddr;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::types::{
    ApiResponse, CertChainItem, DnsLookupRecord, DnsLookupResult, IpGeoInfo, IpLookupResult,
    SslCertInfo, SslCheckResult, WhoisResult,
};

/// 嵌入 WHOIS 服务器配置
const WHOIS_SERVERS: &str = include_str!("../resources/whois_servers.json");

/// WHOIS 查询
#[tauri::command]
pub async fn whois_lookup(domain: String) -> Result<ApiResponse<WhoisResult>, String> {
    let whois =
        WhoIs::from_string(WHOIS_SERVERS).map_err(|e| format!("初始化 WHOIS 客户端失败: {}", e))?;

    let options =
        WhoIsLookupOptions::from_string(&domain).map_err(|e| format!("无效的域名: {}", e))?;

    let raw = whois
        .lookup_async(options)
        .await
        .map_err(|e| format!("WHOIS 查询失败: {}", e))?;

    // 解析原始 WHOIS 数据
    let result = parse_whois_response(&domain, &raw);

    Ok(ApiResponse::success(result))
}

/// 解析 WHOIS 原始响应
fn parse_whois_response(domain: &str, raw: &str) -> WhoisResult {
    WhoisResult {
        domain: domain.to_string(),
        registrar: extract_field(
            raw,
            &[
                r"(?i)Registrar:\s*(.+)",
                r"(?i)Registrar Name:\s*(.+)",
                r"(?i)Sponsoring Registrar:\s*(.+)",
            ],
        ),
        creation_date: extract_field(
            raw,
            &[
                r"(?i)Creation Date:\s*(.+)",
                r"(?i)Created Date:\s*(.+)",
                r"(?i)Created:\s*(.+)",
                r"(?i)Registration Time:\s*(.+)",
                r"(?i)Registration Date:\s*(.+)",
            ],
        ),
        expiration_date: extract_field(
            raw,
            &[
                r"(?i)Expir(?:y|ation) Date:\s*(.+)",
                r"(?i)Registry Expiry Date:\s*(.+)",
                r"(?i)Expiration Time:\s*(.+)",
                r"(?i)paid-till:\s*(.+)",
            ],
        ),
        updated_date: extract_field(
            raw,
            &[
                r"(?i)Updated Date:\s*(.+)",
                r"(?i)Last Updated:\s*(.+)",
                r"(?i)Last Modified:\s*(.+)",
            ],
        ),
        name_servers: extract_name_servers(raw),
        status: extract_status(raw),
        raw: raw.to_string(),
    }
}

/// 使用多个正则模式提取字段
fn extract_field(text: &str, patterns: &[&str]) -> Option<String> {
    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(m) = caps.get(1) {
                    let value = m.as_str().trim().to_string();
                    if !value.is_empty() {
                        return Some(value);
                    }
                }
            }
        }
    }
    None
}

/// 提取域名服务器
fn extract_name_servers(text: &str) -> Vec<String> {
    let mut servers = Vec::new();
    let patterns = [
        r"(?i)Name Server:\s*(.+)",
        r"(?i)nserver:\s*(.+)",
        r"(?i)DNS:\s*(.+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(text) {
                if let Some(m) = caps.get(1) {
                    let server = m.as_str().trim().to_lowercase();
                    if !server.is_empty() && !servers.contains(&server) {
                        servers.push(server);
                    }
                }
            }
        }
    }

    servers
}

/// 提取域名状态
fn extract_status(text: &str) -> Vec<String> {
    let mut statuses = Vec::new();
    let patterns = [
        r"(?i)Domain Status:\s*(.+)",
        r"(?i)Status:\s*(.+)",
        r"(?i)state:\s*(.+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(text) {
                if let Some(m) = caps.get(1) {
                    let status = m.as_str().trim().to_string();
                    // 只取状态名，去掉后面的 URL
                    let status = status
                        .split_whitespace()
                        .next()
                        .unwrap_or(&status)
                        .to_string();
                    if !status.is_empty() && !statuses.contains(&status) {
                        statuses.push(status);
                    }
                }
            }
        }
    }

    statuses
}

/// DNS 查询
#[tauri::command]
pub async fn dns_lookup(
    domain: String,
    record_type: String,
    nameserver: Option<String>,
) -> Result<ApiResponse<DnsLookupResult>, String> {
    // 获取系统默认 DNS 服务器地址的辅助函数
    fn get_system_dns() -> String {
        let config = ResolverConfig::default();
        let servers: Vec<String> = config
            .name_servers()
            .iter()
            .map(|ns| ns.socket_addr.ip().to_string())
            .collect();
        if servers.is_empty() {
            "系统默认".to_string()
        } else {
            servers.join(", ")
        }
    }

    // 根据 nameserver 参数决定使用自定义还是系统默认
    let (resolver, used_nameserver) = if let Some(ref ns) = nameserver {
        if ns.is_empty() {
            // 空字符串视为系统默认
            let system_dns = get_system_dns();
            let provider = TokioConnectionProvider::default();
            let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
                .with_options(ResolverOpts::default())
                .build();
            (resolver, system_dns)
        } else {
            // 解析自定义 nameserver 地址
            let ns_ip: IpAddr = ns
                .parse()
                .map_err(|_| format!("无效的 DNS 服务器地址: {}", ns))?;

            let config = ResolverConfig::from_parts(
                None,
                vec![],
                NameServerConfigGroup::from_ips_clear(&[ns_ip], 53, true),
            );
            let provider = TokioConnectionProvider::default();
            let resolver = TokioResolver::builder_with_config(config, provider)
                .with_options(ResolverOpts::default())
                .build();
            (resolver, ns.clone())
        }
    } else {
        // 使用系统默认
        let system_dns = get_system_dns();
        let provider = TokioConnectionProvider::default();
        let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
            .with_options(ResolverOpts::default())
            .build();
        (resolver, system_dns)
    };

    let mut records: Vec<DnsLookupRecord> = Vec::new();
    let record_type_upper = record_type.to_uppercase();

    match record_type_upper.as_str() {
        "A" => {
            if let Ok(response) = resolver.ipv4_lookup(&domain).await {
                for ip in response.iter() {
                    records.push(DnsLookupRecord {
                        record_type: "A".to_string(),
                        name: domain.clone(),
                        value: ip.to_string(),
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: None,
                    });
                }
            }
        }
        "AAAA" => {
            if let Ok(response) = resolver.ipv6_lookup(&domain).await {
                for ip in response.iter() {
                    records.push(DnsLookupRecord {
                        record_type: "AAAA".to_string(),
                        name: domain.clone(),
                        value: ip.to_string(),
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: None,
                    });
                }
            }
        }
        "MX" => {
            if let Ok(response) = resolver.mx_lookup(&domain).await {
                for mx in response.iter() {
                    records.push(DnsLookupRecord {
                        record_type: "MX".to_string(),
                        name: domain.clone(),
                        value: mx.exchange().to_string().trim_end_matches('.').to_string(),
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: Some(mx.preference()),
                    });
                }
            }
        }
        "TXT" => {
            if let Ok(response) = resolver.txt_lookup(&domain).await {
                for txt in response.iter() {
                    let txt_data: String = txt
                        .iter()
                        .map(|data| String::from_utf8_lossy(data).to_string())
                        .collect::<Vec<_>>()
                        .join("");
                    records.push(DnsLookupRecord {
                        record_type: "TXT".to_string(),
                        name: domain.clone(),
                        value: txt_data,
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: None,
                    });
                }
            }
        }
        "NS" => {
            if let Ok(response) = resolver.ns_lookup(&domain).await {
                for ns in response.iter() {
                    records.push(DnsLookupRecord {
                        record_type: "NS".to_string(),
                        name: domain.clone(),
                        value: ns.to_string().trim_end_matches('.').to_string(),
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: None,
                    });
                }
            }
        }
        "CNAME" => {
            if let Ok(response) = resolver
                .lookup(&domain, hickory_resolver::proto::rr::RecordType::CNAME)
                .await
            {
                for record in response.record_iter() {
                    if let Some(cname) = record.data().as_cname() {
                        records.push(DnsLookupRecord {
                            record_type: "CNAME".to_string(),
                            name: domain.clone(),
                            value: cname.0.to_string().trim_end_matches('.').to_string(),
                            ttl: record.ttl(),
                            priority: None,
                        });
                    }
                }
            }
        }
        "SOA" => {
            if let Ok(response) = resolver.soa_lookup(&domain).await {
                if let Some(soa) = response.iter().next() {
                    let value = format!(
                        "{} {} {} {} {} {} {}",
                        soa.mname().to_string().trim_end_matches('.'),
                        soa.rname().to_string().trim_end_matches('.'),
                        soa.serial(),
                        soa.refresh(),
                        soa.retry(),
                        soa.expire(),
                        soa.minimum()
                    );
                    records.push(DnsLookupRecord {
                        record_type: "SOA".to_string(),
                        name: domain.clone(),
                        value,
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: None,
                    });
                }
            }
        }
        "SRV" => {
            if let Ok(response) = resolver.srv_lookup(&domain).await {
                for srv in response.iter() {
                    let value = format!(
                        "{} {} {}",
                        srv.weight(),
                        srv.port(),
                        srv.target().to_string().trim_end_matches('.')
                    );
                    records.push(DnsLookupRecord {
                        record_type: "SRV".to_string(),
                        name: domain.clone(),
                        value,
                        ttl: response
                            .as_lookup()
                            .record_iter()
                            .next()
                            .map(|r| r.ttl())
                            .unwrap_or(0),
                        priority: Some(srv.priority()),
                    });
                }
            }
        }
        "CAA" => {
            if let Ok(response) = resolver
                .lookup(&domain, hickory_resolver::proto::rr::RecordType::CAA)
                .await
            {
                for record in response.record_iter() {
                    if let Some(caa) = record.data().as_caa() {
                        let value = format!(
                            "{} {} \"{}\"",
                            if caa.issuer_critical() { 128 } else { 0 },
                            caa.tag().as_str(),
                            String::from_utf8_lossy(caa.raw_value())
                        );
                        records.push(DnsLookupRecord {
                            record_type: "CAA".to_string(),
                            name: domain.clone(),
                            value,
                            ttl: record.ttl(),
                            priority: None,
                        });
                    }
                }
            }
        }
        "PTR" => {
            if let Ok(response) = resolver
                .lookup(&domain, hickory_resolver::proto::rr::RecordType::PTR)
                .await
            {
                for record in response.record_iter() {
                    if let Some(ptr) = record.data().as_ptr() {
                        records.push(DnsLookupRecord {
                            record_type: "PTR".to_string(),
                            name: domain.clone(),
                            value: ptr.0.to_string().trim_end_matches('.').to_string(),
                            ttl: record.ttl(),
                            priority: None,
                        });
                    }
                }
            }
        }
        "ALL" => {
            // 并发查询所有记录类型
            let types = vec![
                "A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA", "SRV", "CAA", "PTR",
            ];
            let ns = nameserver.clone();
            let futures: Vec<_> = types
                .into_iter()
                .map(|t| Box::pin(dns_lookup(domain.clone(), t.to_string(), ns.clone())))
                .collect();

            let results = join_all(futures).await;
            for result in results {
                if let Ok(ApiResponse {
                    data: Some(lookup_result),
                    ..
                }) = result
                {
                    records.extend(lookup_result.records);
                }
            }
        }
        _ => {
            return Err(format!("不支持的记录类型: {}", record_type));
        }
    }

    Ok(ApiResponse::success(DnsLookupResult {
        nameserver: used_nameserver,
        records,
    }))
}

/// ipwhois.io 响应结构
#[derive(serde::Deserialize)]
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

#[derive(serde::Deserialize)]
struct IpWhoisTimezone {
    id: Option<String>,
}

#[derive(serde::Deserialize)]
struct IpWhoisConnection {
    asn: Option<i64>,
    org: Option<String>,
    isp: Option<String>,
}

/// 查询单个 IP 的地理位置
async fn lookup_single_ip(ip: &str, client: &reqwest::Client) -> Result<IpGeoInfo, String> {
    let url = format!(
        "https://ipwho.is/{}?fields=ip,success,message,type,country,country_code,region,city,latitude,longitude,timezone,connection",
        ip
    );

    let response: IpWhoisResponse = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析失败: {}", e))?;

    if !response.success {
        let error_msg = match response.message.as_deref() {
            Some("You've hit the monthly limit") => {
                "IP 查询服务已达本月限额，请稍后再试".to_string()
            }
            Some("Invalid IP address") => "无效的 IP 地址".to_string(),
            Some("Reserved range") => "该 IP 属于保留地址段，无法查询".to_string(),
            Some(msg) => format!("查询失败: {}", msg),
            None => "查询失败".to_string(),
        };
        return Err(error_msg);
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
            conn.asn.map(|n| format!("AS{}", n)),
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
/// 支持直接输入 IP 地址或域名，域名会解析出所有 IPv4/IPv6 地址
#[tauri::command]
pub async fn ip_lookup(query: String) -> Result<ApiResponse<IpLookupResult>, String> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err("请输入 IP 地址或域名".to_string());
    }

    let client = reqwest::Client::new();

    // 检查是否为 IP 地址
    if let Ok(_ip_addr) = query.parse::<std::net::IpAddr>() {
        // 直接查询 IP
        let result = lookup_single_ip(&query, &client).await?;
        return Ok(ApiResponse::success(IpLookupResult {
            query,
            is_domain: false,
            results: vec![result],
        }));
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
        return Err(format!("无法解析域名: {}", query));
    }

    // 查询每个 IP 的地理位置（并行）
    let mut results = Vec::new();
    for ip in ips {
        match lookup_single_ip(&ip, &client).await {
            Ok(info) => results.push(info),
            Err(e) => {
                // 记录错误但继续处理其他 IP
                eprintln!("查询 IP {} 失败: {}", ip, e);
            }
        }
    }

    if results.is_empty() {
        return Err("所有 IP 地址查询均失败".to_string());
    }

    Ok(ApiResponse::success(IpLookupResult {
        query,
        is_domain: true,
        results,
    }))
}

/// 检查 HTTP 连接是否可用
fn check_http_connection(domain: &str, port: u16) -> bool {
    use std::io::{Read, Write};
    use std::net::TcpStream;

    if let Ok(mut stream) = TcpStream::connect(format!("{}:{}", domain, port)) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .ok();
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .ok();

        let request = format!(
            "HEAD / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            domain
        );

        if stream.write_all(request.as_bytes()).is_ok() {
            let mut response = vec![0u8; 128];
            if stream.read(&mut response).is_ok() {
                let response_str = String::from_utf8_lossy(&response);
                // 检查是否是 HTTP 响应
                return response_str.starts_with("HTTP/");
            }
        }
    }
    false
}

/// SSL 证书检查（桌面端使用 native-tls）
/// 支持自定义端口，如果 HTTPS 连接失败会回退检测 HTTP
#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn ssl_check(
    domain: String,
    port: Option<u16>,
) -> Result<ApiResponse<SslCheckResult>, String> {
    use native_tls::TlsConnector;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use x509_parser::prelude::*;

    let port = port.unwrap_or(443);
    let domain_clone = domain.clone();

    tokio::task::spawn_blocking(move || {
        // 尝试建立 TCP 连接
        let stream = match TcpStream::connect(format!("{}:{}", domain_clone, port)) {
            Ok(s) => s,
            Err(e) => {
                // 连接失败
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("连接失败: {}", e)),
                }));
            }
        };
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .ok();

        // 尝试建立 TLS 连接
        let connector = match TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("TLS 初始化失败: {}", e)),
                }));
            }
        };

        let mut tls_stream = match connector.connect(&domain_clone, stream) {
            Ok(s) => s,
            Err(_) => {
                // TLS 握手失败，检测是否是 HTTP 连接
                if check_http_connection(&domain_clone, port) {
                    return Ok(ApiResponse::success(SslCheckResult {
                        domain: domain_clone,
                        port,
                        connection_status: "http".to_string(),
                        cert_info: None,
                        error: None,
                    }));
                }
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some("TLS 握手失败，且非 HTTP 连接".to_string()),
                }));
            }
        };

        // 发送 HTTP 请求
        let request = format!(
            "HEAD / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            domain_clone
        );
        tls_stream.write_all(request.as_bytes()).ok();
        let mut response = vec![0u8; 1024];
        tls_stream.read(&mut response).ok();

        // 获取证书
        let cert_chain = match tls_stream.peer_certificate() {
            Ok(Some(c)) => c,
            _ => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some("未找到证书".to_string()),
                }));
            }
        };

        let cert_der = match cert_chain.to_der() {
            Ok(d) => d,
            Err(e) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书编码失败: {}", e)),
                }));
            }
        };

        // 解析证书
        let (_, cert) = match X509Certificate::from_der(&cert_der) {
            Ok(c) => c,
            Err(e) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书解析失败: {}", e)),
                }));
            }
        };

        // 提取信息
        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();
        let valid_from = cert.validity().not_before.to_rfc2822().unwrap_or_default();
        let valid_to = cert.validity().not_after.to_rfc2822().unwrap_or_default();

        // 计算剩余天数
        let now = chrono::Utc::now();
        let not_after = chrono::DateTime::parse_from_rfc2822(&valid_to)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        let days_remaining = (not_after - now).num_days();
        let is_expired = days_remaining < 0;

        // 验证证书是否有效
        let is_valid = TlsConnector::new()
            .map(|c| {
                TcpStream::connect(format!("{}:{}", domain_clone, port))
                    .ok()
                    .and_then(|s| c.connect(&domain_clone, s).ok())
                    .is_some()
            })
            .unwrap_or(false);

        // 提取 SAN
        let san: Vec<String> = cert
            .subject_alternative_name()
            .ok()
            .flatten()
            .map(|ext| {
                ext.value
                    .general_names
                    .iter()
                    .filter_map(|name| match name {
                        x509_parser::extensions::GeneralName::DNSName(dns) => {
                            Some((*dns).to_string())
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let serial_number = cert.serial.to_str_radix(16).to_uppercase();
        let signature_algorithm = cert.signature_algorithm.algorithm.to_string();

        let certificate_chain = vec![CertChainItem {
            subject: subject.clone(),
            issuer: issuer.clone(),
            is_ca: cert.is_ca(),
        }];

        Ok(ApiResponse::success(SslCheckResult {
            domain: domain_clone.clone(),
            port,
            connection_status: "https".to_string(),
            cert_info: Some(SslCertInfo {
                domain: domain_clone,
                issuer,
                subject,
                valid_from,
                valid_to,
                days_remaining,
                is_expired,
                is_valid,
                san,
                serial_number,
                signature_algorithm,
                certificate_chain,
            }),
            error: None,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}

/// SSL 证书检查（Android 使用 rustls）
#[cfg(target_os = "android")]
#[tauri::command]
pub async fn ssl_check(
    domain: String,
    port: Option<u16>,
) -> Result<ApiResponse<SslCheckResult>, String> {
    use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::Arc;
    use x509_parser::prelude::*;

    let port = port.unwrap_or(443);
    let domain_clone = domain.clone();

    tokio::task::spawn_blocking(move || {
        // 尝试建立 TCP 连接
        let stream = match TcpStream::connect(format!("{}:{}", domain_clone, port)) {
            Ok(s) => s,
            Err(e) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("连接失败: {}", e)),
                }));
            }
        };
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .ok();

        // 配置 rustls
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = match domain_clone.clone().try_into() {
            Ok(n) => n,
            Err(_) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some("无效的域名".to_string()),
                }));
            }
        };

        let conn = match ClientConnection::new(Arc::new(config), server_name) {
            Ok(c) => c,
            Err(e) => {
                // TLS 连接失败，检测是否是 HTTP 连接
                if check_http_connection(&domain_clone, port) {
                    return Ok(ApiResponse::success(SslCheckResult {
                        domain: domain_clone,
                        port,
                        connection_status: "http".to_string(),
                        cert_info: None,
                        error: None,
                    }));
                }
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("TLS 初始化失败: {}", e)),
                }));
            }
        };

        let mut tls_stream = StreamOwned::new(conn, stream);

        // 发送请求触发握手
        let request = format!(
            "HEAD / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            domain_clone
        );
        if tls_stream.write_all(request.as_bytes()).is_err() {
            // 写入失败，检测是否是 HTTP 连接
            if check_http_connection(&domain_clone, port) {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "http".to_string(),
                    cert_info: None,
                    error: None,
                }));
            }
            return Ok(ApiResponse::success(SslCheckResult {
                domain: domain_clone,
                port,
                connection_status: "failed".to_string(),
                cert_info: None,
                error: Some("TLS 握手失败".to_string()),
            }));
        }
        let mut response = vec![0u8; 1024];
        tls_stream.read(&mut response).ok();

        // 获取证书
        let certs = match tls_stream.conn.peer_certificates() {
            Some(c) if !c.is_empty() => c,
            _ => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some("未找到证书".to_string()),
                }));
            }
        };

        let cert_der = certs[0].as_ref();

        // 解析证书
        let (_, cert) = match X509Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(e) => {
                return Ok(ApiResponse::success(SslCheckResult {
                    domain: domain_clone,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书解析失败: {}", e)),
                }));
            }
        };

        // 提取信息
        let subject = cert.subject().to_string();
        let issuer = cert.issuer().to_string();
        let valid_from = cert.validity().not_before.to_rfc2822().unwrap_or_default();
        let valid_to = cert.validity().not_after.to_rfc2822().unwrap_or_default();

        // 计算剩余天数
        let now = chrono::Utc::now();
        let not_after = chrono::DateTime::parse_from_rfc2822(&valid_to)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or(now);
        let days_remaining = (not_after - now).num_days();
        let is_expired = days_remaining < 0;
        let is_valid = !is_expired;

        // 提取 SAN
        let san: Vec<String> = cert
            .subject_alternative_name()
            .ok()
            .flatten()
            .map(|ext| {
                ext.value
                    .general_names
                    .iter()
                    .filter_map(|name| match name {
                        x509_parser::extensions::GeneralName::DNSName(dns) => {
                            Some((*dns).to_string())
                        }
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let serial_number = cert.serial.to_str_radix(16).to_uppercase();
        let signature_algorithm = cert.signature_algorithm.algorithm.to_string();

        let certificate_chain: Vec<CertChainItem> = certs
            .iter()
            .filter_map(|c| {
                X509Certificate::from_der(c.as_ref())
                    .ok()
                    .map(|(_, parsed)| CertChainItem {
                        subject: parsed.subject().to_string(),
                        issuer: parsed.issuer().to_string(),
                        is_ca: parsed.is_ca(),
                    })
            })
            .collect();

        Ok(ApiResponse::success(SslCheckResult {
            domain: domain_clone.clone(),
            port,
            connection_status: "https".to_string(),
            cert_info: Some(SslCertInfo {
                domain: domain_clone,
                issuer,
                subject,
                valid_from,
                valid_to,
                days_remaining,
                is_expired,
                is_valid,
                san,
                serial_number,
                signature_algorithm,
                certificate_chain,
            }),
            error: None,
        }))
    })
    .await
    .map_err(|e| format!("任务执行失败: {}", e))?
}
