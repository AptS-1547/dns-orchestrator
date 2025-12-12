//! SSL 证书检查模块
//!
//! 根据 feature flag 选择不同的 TLS 实现：
//! - `native-tls`: 桌面端使用系统 TLS
//! - `rustls`: Android/Web 使用纯 Rust TLS

use std::io::{Read, Write};
use std::net::TcpStream;

use crate::error::{CoreError, CoreResult};
use crate::types::{CertChainItem, SslCertInfo, SslCheckResult};

/// 检查 HTTP 连接是否可用
fn check_http_connection(domain: &str, port: u16) -> bool {
    if let Ok(mut stream) = TcpStream::connect(format!("{domain}:{port}")) {
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .ok();
        stream
            .set_write_timeout(Some(std::time::Duration::from_secs(5)))
            .ok();

        let request = format!("HEAD / HTTP/1.1\r\nHost: {domain}\r\nConnection: close\r\n\r\n");

        if stream.write_all(request.as_bytes()).is_ok() {
            let mut response = vec![0u8; 128];
            if stream.read(&mut response).is_ok() {
                let response_str = String::from_utf8_lossy(&response);
                return response_str.starts_with("HTTP/");
            }
        }
    }
    false
}

/// SSL 证书检查（使用 native-tls）
#[cfg(feature = "native-tls")]
pub async fn ssl_check(domain: &str, port: Option<u16>) -> CoreResult<SslCheckResult> {
    use native_tls_crate::TlsConnector;
    use x509_parser::prelude::*;

    let port = port.unwrap_or(443);
    let domain = domain.to_string();

    tokio::task::spawn_blocking(move || {
        // 尝试建立 TCP 连接
        let stream = match TcpStream::connect(format!("{domain}:{port}")) {
            Ok(s) => s,
            Err(e) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("连接失败: {e}")),
                });
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
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("TLS 初始化失败: {e}")),
                });
            }
        };

        let Ok(mut tls_stream) = connector.connect(&domain, stream) else {
            if check_http_connection(&domain, port) {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "http".to_string(),
                    cert_info: None,
                    error: None,
                });
            }
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "failed".to_string(),
                cert_info: None,
                error: Some("TLS 握手失败，且非 HTTP 连接".to_string()),
            });
        };

        // 发送 HTTP 请求
        let request = format!("HEAD / HTTP/1.1\r\nHost: {domain}\r\nConnection: close\r\n\r\n");
        tls_stream.write_all(request.as_bytes()).ok();
        let mut response = vec![0u8; 1024];
        tls_stream.read(&mut response).ok();

        // 获取证书
        let Ok(Some(cert_chain)) = tls_stream.peer_certificate() else {
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "https".to_string(),
                cert_info: None,
                error: Some("未找到证书".to_string()),
            });
        };

        let cert_der = match cert_chain.to_der() {
            Ok(d) => d,
            Err(e) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书编码失败: {e}")),
                });
            }
        };

        // 解析证书
        let (_, cert) = match X509Certificate::from_der(&cert_der) {
            Ok(c) => c,
            Err(e) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书解析失败: {e}")),
                });
            }
        };

        let cert_info = parse_certificate(&domain, port, &cert);

        Ok(SslCheckResult {
            domain: domain.clone(),
            port,
            connection_status: "https".to_string(),
            cert_info: Some(cert_info),
            error: None,
        })
    })
    .await
    .map_err(|e| CoreError::NetworkError(format!("任务执行失败: {e}")))?
}

/// SSL 证书检查（使用 rustls）
#[cfg(all(feature = "rustls", not(feature = "native-tls")))]
pub async fn ssl_check(domain: &str, port: Option<u16>) -> CoreResult<SslCheckResult> {
    use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
    use std::sync::Arc;
    use x509_parser::prelude::*;

    let port = port.unwrap_or(443);
    let domain = domain.to_string();

    tokio::task::spawn_blocking(move || {
        // 尝试建立 TCP 连接
        let stream = match TcpStream::connect(format!("{domain}:{port}")) {
            Ok(s) => s,
            Err(e) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("连接失败: {e}")),
                });
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

        let server_name = match domain.clone().try_into() {
            Ok(n) => n,
            Err(_) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some("无效的域名".to_string()),
                });
            }
        };

        let conn = match ClientConnection::new(Arc::new(config), server_name) {
            Ok(c) => c,
            Err(e) => {
                if check_http_connection(&domain, port) {
                    return Ok(SslCheckResult {
                        domain,
                        port,
                        connection_status: "http".to_string(),
                        cert_info: None,
                        error: None,
                    });
                }
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "failed".to_string(),
                    cert_info: None,
                    error: Some(format!("TLS 初始化失败: {e}")),
                });
            }
        };

        let mut tls_stream = StreamOwned::new(conn, stream);

        // 发送请求触发握手
        let request = format!("HEAD / HTTP/1.1\r\nHost: {domain}\r\nConnection: close\r\n\r\n");
        if tls_stream.write_all(request.as_bytes()).is_err() {
            if check_http_connection(&domain, port) {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "http".to_string(),
                    cert_info: None,
                    error: None,
                });
            }
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "failed".to_string(),
                cert_info: None,
                error: Some("TLS 握手失败".to_string()),
            });
        }
        let mut response = vec![0u8; 1024];
        tls_stream.read(&mut response).ok();

        // 获取证书
        let certs = match tls_stream.conn.peer_certificates() {
            Some(c) if !c.is_empty() => c,
            _ => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some("未找到证书".to_string()),
                });
            }
        };

        let cert_der = certs[0].as_ref();

        // 解析证书
        let (_, cert) = match X509Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(e) => {
                return Ok(SslCheckResult {
                    domain,
                    port,
                    connection_status: "https".to_string(),
                    cert_info: None,
                    error: Some(format!("证书解析失败: {e}")),
                });
            }
        };

        let mut cert_info = parse_certificate(&domain, port, &cert);

        // 解析完整证书链
        cert_info.certificate_chain = certs
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

        Ok(SslCheckResult {
            domain: domain.clone(),
            port,
            connection_status: "https".to_string(),
            cert_info: Some(cert_info),
            error: None,
        })
    })
    .await
    .map_err(|e| CoreError::NetworkError(format!("任务执行失败: {e}")))?
}

/// 解析证书信息
#[cfg(any(feature = "native-tls", feature = "rustls"))]
fn parse_certificate(
    query: &str,
    _port: u16,
    cert: &x509_parser::certificate::X509Certificate,
) -> SslCertInfo {
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
                    x509_parser::extensions::GeneralName::DNSName(dns) => Some((*dns).to_string()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    // 从证书提取 CN
    let cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .map(String::from);

    // 从证书提取实际域名
    // 优先级: CN > SAN 第一个 > 用户查询值
    let cert_domain = cn
        .clone()
        .or_else(|| san.first().cloned())
        .unwrap_or_else(|| query.to_string());

    // 检查域名是否匹配（CN 或 SAN 中任意一个）
    let domain_matches = check_domain_match(query, cn.as_deref(), &san);

    // is_valid = 未过期 且 域名匹配
    let is_valid = !is_expired && domain_matches;

    let serial_number = cert.serial.to_str_radix(16).to_uppercase();
    let signature_algorithm = cert.signature_algorithm.algorithm.to_string();

    let certificate_chain = vec![CertChainItem {
        subject: subject.clone(),
        issuer: issuer.clone(),
        is_ca: cert.is_ca(),
    }];

    SslCertInfo {
        domain: cert_domain,
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
    }
}

/// 检查查询的域名/IP 是否与证书的 CN 或 SAN 匹配
#[cfg(any(feature = "native-tls", feature = "rustls"))]
fn check_domain_match(query: &str, cn: Option<&str>, san: &[String]) -> bool {
    let query_lower = query.to_lowercase();

    // 检查 CN
    if let Some(cn) = cn {
        if matches_domain(&query_lower, &cn.to_lowercase()) {
            return true;
        }
    }

    // 检查 SAN
    for name in san {
        if matches_domain(&query_lower, &name.to_lowercase()) {
            return true;
        }
    }

    false
}

/// 域名匹配（支持通配符）
#[cfg(any(feature = "native-tls", feature = "rustls"))]
fn matches_domain(query: &str, pattern: &str) -> bool {
    // 精确匹配
    if query == pattern {
        return true;
    }

    // 通配符匹配 (*.example.com)
    if let Some(suffix) = pattern.strip_prefix("*.") {
        // 通配符只匹配一级子域名
        // 例如: *.example.com 匹配 foo.example.com，但不匹配 foo.bar.example.com
        if let Some(prefix) = query.strip_suffix(suffix) {
            // prefix 应该是 "xxx." 的形式，且 xxx 中不能包含 "."
            if prefix.ends_with('.') && !prefix[..prefix.len() - 1].contains('.') {
                return true;
            }
        }
    }

    false
}

/// 无 TLS 支持时的 SSL 检查（返回错误）
#[cfg(not(any(feature = "native-tls", feature = "rustls")))]
pub async fn ssl_check(_domain: &str, _port: Option<u16>) -> CoreResult<SslCheckResult> {
    Err(CoreError::ValidationError(
        "SSL 检查功能未启用，请编译时启用 native-tls 或 rustls feature".to_string(),
    ))
}
