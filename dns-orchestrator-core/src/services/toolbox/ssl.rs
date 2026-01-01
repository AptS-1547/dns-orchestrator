//! SSL 证书检查模块
//!
//! 使用 rustls 实现纯异步的 SSL 证书检查，支持完整证书链获取

use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, trace, warn};
use rustls::crypto::CryptoProvider;
use rustls::{ClientConfig, RootCertStore};
use rustls_pki_types::{CertificateDer, ServerName};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;
use x509_parser::prelude::*;

use crate::error::CoreResult;
use crate::types::{CertChainItem, SslCertInfo, SslCheckResult};

// 超时配置常量
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const TLS_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_TIMEOUT: Duration = Duration::from_secs(3);

/// 初始化 rustls CryptoProvider（仅初始化一次）
fn ensure_crypto_provider() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        if let Err(e) = CryptoProvider::install_default(rustls::crypto::ring::default_provider()) {
            error!(
                "FATAL: Failed to install rustls crypto provider: {e:?}. \
                 This is a critical initialization error. The application cannot continue."
            );
            panic!("Failed to install default rustls crypto provider: {e:?}");
        }
    });
}

/// 检查 HTTP 连接是否可用（异步版本）
async fn check_http_connection(domain: &str, port: u16) -> bool {
    // 使用 timeout 包装整个 HTTP 检测过程
    let result = timeout(HTTP_TIMEOUT, async {
        // 建立 TCP 连接
        let mut stream = timeout(
            CONNECT_TIMEOUT,
            TcpStream::connect(format!("{domain}:{port}")),
        )
        .await
        .ok()?
        .ok()?;

        // 发送 HTTP HEAD 请求
        let request = format!("HEAD / HTTP/1.1\r\nHost: {domain}\r\nConnection: close\r\n\r\n");
        stream.write_all(request.as_bytes()).await.ok()?;

        // 读取响应
        let mut response = vec![0u8; 128];
        let _ = stream.read(&mut response).await.ok()?;

        let response_str = String::from_utf8_lossy(&response);
        Some(response_str.starts_with("HTTP/"))
    })
    .await;

    result.unwrap_or(None).unwrap_or(false)
}

/// SSL 证书检查（使用 rustls 纯异步实现）
#[cfg(feature = "rustls")]
pub async fn ssl_check(domain: &str, port: Option<u16>) -> CoreResult<SslCheckResult> {
    // 确保 CryptoProvider 已初始化
    ensure_crypto_provider();

    let port = port.unwrap_or(443);
    let domain = domain.to_string();

    debug!("[SSL] Starting check for {}:{}", domain, port);
    let start_time = std::time::Instant::now();

    // 1. 建立 TCP 连接（带超时）
    trace!("[SSL] Establishing TCP connection...");
    let stream = match timeout(
        CONNECT_TIMEOUT,
        TcpStream::connect(format!("{domain}:{port}")),
    )
    .await
    {
        Ok(Ok(s)) => {
            trace!(
                "[SSL] TCP connection succeeded, took {:?}",
                start_time.elapsed()
            );
            s
        }
        Ok(Err(e)) => {
            warn!("[SSL] TCP connection failed: {}", e);
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "failed".to_string(),
                cert_info: None,
                error: Some(format!("连接失败: {e}")),
            });
        }
        Err(_) => {
            warn!(
                "[SSL] TCP connection timeout ({}s)",
                CONNECT_TIMEOUT.as_secs()
            );
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "failed".to_string(),
                cert_info: None,
                error: Some("连接超时".to_string()),
            });
        }
    };

    // 2. 配置 rustls 客户端
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));

    let Ok(server_name) = ServerName::try_from(domain.clone()) else {
        warn!("[SSL] Invalid domain name: {}", domain);
        return Ok(SslCheckResult {
            domain,
            port,
            connection_status: "failed".to_string(),
            cert_info: None,
            error: Some("无效的域名".to_string()),
        });
    };

    // 3. TLS 握手（带超时）
    trace!("[SSL] Performing TLS handshake...");
    let tls_start = std::time::Instant::now();
    let tls_result = timeout(TLS_TIMEOUT, connector.connect(server_name, stream)).await;

    let tls_stream = match tls_result {
        Ok(Ok(stream)) => {
            trace!(
                "[SSL] TLS handshake succeeded, took {:?}",
                tls_start.elapsed()
            );
            stream
        }
        Ok(Err(e)) => {
            warn!("[SSL] TLS handshake failed: {}", e);
            // TLS 握手失败，检查是否为 HTTP
            trace!("[SSL] Checking if HTTP connection...");
            if check_http_connection(&domain, port).await {
                debug!(
                    "[SSL] Detected HTTP connection, total time {:?}",
                    start_time.elapsed()
                );
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
                error: Some(format!("TLS 握手失败: {e}")),
            });
        }
        Err(_) => {
            warn!("[SSL] TLS handshake timeout ({}s)", TLS_TIMEOUT.as_secs());
            // 超时
            trace!("[SSL] Checking if HTTP connection...");
            if check_http_connection(&domain, port).await {
                debug!(
                    "[SSL] Detected HTTP connection, total time {:?}",
                    start_time.elapsed()
                );
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
                error: Some("TLS 握手超时".to_string()),
            });
        }
    };

    // 4. 获取证书链
    trace!("[SSL] Retrieving certificate chain...");
    let (_, tls_conn) = tls_stream.get_ref();
    let certs = match tls_conn.peer_certificates() {
        Some(c) if !c.is_empty() => {
            trace!("[SSL] Retrieved {} certificate(s)", c.len());
            c
        }
        _ => {
            warn!("[SSL] No certificates found");
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

    // 5. 解析叶子证书
    trace!("[SSL] Parsing certificate...");
    let (_, cert) = match X509Certificate::from_der(cert_der) {
        Ok(c) => c,
        Err(e) => {
            warn!("[SSL] Certificate parsing failed: {}", e);
            return Ok(SslCheckResult {
                domain,
                port,
                connection_status: "https".to_string(),
                cert_info: None,
                error: Some(format!("证书解析失败: {e}")),
            });
        }
    };

    // 6. 解析证书信息
    let mut cert_info = parse_certificate(&domain, port, &cert);

    // 7. 解析完整证书链
    cert_info.certificate_chain = certs
        .iter()
        .filter_map(|c: &CertificateDer| {
            X509Certificate::from_der(c.as_ref())
                .ok()
                .map(|(_, parsed)| CertChainItem {
                    subject: parsed.subject().to_string(),
                    issuer: parsed.issuer().to_string(),
                    is_ca: parsed.is_ca(),
                })
        })
        .collect();

    debug!(
        "[SSL] Check completed: {} - valid={}, expired={}, days_remaining={}, chain_length={}, total_time={:?}",
        domain,
        cert_info.is_valid,
        cert_info.is_expired,
        cert_info.days_remaining,
        cert_info.certificate_chain.len(),
        start_time.elapsed()
    );

    Ok(SslCheckResult {
        domain: domain.clone(),
        port,
        connection_status: "https".to_string(),
        cert_info: Some(cert_info),
        error: None,
    })
}

/// 解析证书信息
#[cfg(feature = "rustls")]
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

    // 证书链将在 ssl_check 函数中填充，这里先初始化为空
    let certificate_chain = vec![];

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
#[cfg(feature = "rustls")]
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
#[cfg(feature = "rustls")]
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

/// 无 rustls 支持时的 SSL 检查（返回错误）
#[cfg(not(feature = "rustls"))]
pub async fn ssl_check(_domain: &str, _port: Option<u16>) -> CoreResult<SslCheckResult> {
    Err(CoreError::ValidationError(
        "SSL 检查功能未启用，请编译时启用 rustls feature".to_string(),
    ))
}
