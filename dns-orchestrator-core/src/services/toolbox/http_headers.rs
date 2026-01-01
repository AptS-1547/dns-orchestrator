//! HTTP 头检查模块

use std::time::Instant;

use log::debug;
use reqwest::{Client, Method};
use url::Url;

use crate::error::{CoreError, CoreResult};
use crate::types::{
    HttpHeader, HttpHeaderCheckRequest, HttpHeaderCheckResult, HttpMethod, SecurityHeaderAnalysis,
};

const REQUEST_TIMEOUT_SECS: u64 = 10;

/// 必需的安全头列表
const REQUIRED_SECURITY_HEADERS: &[&str] = &[
    "strict-transport-security",
    "x-frame-options",
    "x-content-type-options",
    "content-security-policy",
];

/// 建议的安全头列表
const RECOMMENDED_SECURITY_HEADERS: &[&str] =
    &["referrer-policy", "permissions-policy", "x-xss-protection"];

/// HTTP 头检查
pub async fn http_header_check(
    request: &HttpHeaderCheckRequest,
) -> CoreResult<HttpHeaderCheckResult> {
    debug!("[HTTP] Checking headers for {}", request.url);
    let start = Instant::now();

    // 确保 URL 包含协议，如果没有则默认添加 https://
    let url = if request.url.starts_with("http://") || request.url.starts_with("https://") {
        request.url.clone()
    } else {
        format!("https://{}", request.url)
    };

    debug!("[HTTP] Normalized URL: {}", url);

    // 构建 HTTP 客户端
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| CoreError::NetworkError(format!("HTTP client initialization failed: {e}")))?;

    // 转换 HTTP 方法
    let method = match request.method {
        HttpMethod::GET => Method::GET,
        HttpMethod::HEAD => Method::HEAD,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::PATCH => Method::PATCH,
        HttpMethod::OPTIONS => Method::OPTIONS,
    };

    // 构建请求
    let mut req_builder = client.request(method.clone(), &url);

    // 添加自定义请求头
    for header in &request.custom_headers {
        req_builder = req_builder.header(&header.name, &header.value);
    }

    // 添加请求体（POST/PUT/PATCH）
    if let Some(body) = &request.body {
        if let Some(content_type) = &request.content_type {
            req_builder = req_builder.header("Content-Type", content_type);
        }
        req_builder = req_builder.body(body.clone());
    }

    // 发送请求
    let response = req_builder
        .send()
        .await
        .map_err(|e| CoreError::NetworkError(format!("HTTP request failed: {e}")))?;

    let elapsed = start.elapsed();
    let status_code = response.status().as_u16();
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("Unknown")
        .to_string();

    // 提取响应头
    let mut headers: Vec<HttpHeader> = Vec::new();
    for (name, value) in response.headers().iter() {
        headers.push(HttpHeader {
            name: name.to_string(),
            value: value.to_str().unwrap_or("<binary>").to_string(),
        });
    }

    // 读取响应体为 bytes
    let body_bytes = response
        .bytes()
        .await
        .map_err(|e| CoreError::NetworkError(format!("Failed to read response body: {e}")))?;

    // Content-Length - 计算实际大小
    let content_length = Some(body_bytes.len() as u64);

    // 将 body_bytes 转换为字符串（用于 raw_response）
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    // 安全头分析
    let security_analysis = analyze_security_headers(&headers);

    // 构建原始请求报文
    let mut raw_request = format!("{} {} HTTP/1.1\r\n", method.as_str(), url);

    // 使用 url crate 解析 Host
    if let Ok(parsed_url) = Url::parse(&url) {
        if let Some(host) = parsed_url.host_str() {
            let host_header = if let Some(port) = parsed_url.port() {
                format!("{host}:{port}")
            } else {
                host.to_string()
            };
            raw_request.push_str(&format!("Host: {host_header}\r\n"));
        }
    }

    // 添加自定义请求头
    for header in &request.custom_headers {
        if !header.name.is_empty() && !header.value.is_empty() {
            raw_request.push_str(&format!("{}: {}\r\n", header.name, header.value));
        }
    }

    // 添加 Content-Type 和请求体
    if let Some(body) = &request.body {
        if let Some(content_type) = &request.content_type {
            raw_request.push_str(&format!("Content-Type: {}\r\n", content_type));
        }
        raw_request.push_str(&format!("Content-Length: {}\r\n", body.len()));
        raw_request.push_str("\r\n");
        raw_request.push_str(body);
    } else {
        raw_request.push_str("\r\n");
    }

    // 构建原始响应报文
    let mut raw_response = format!("HTTP/1.1 {} {}\r\n", status_code, status_text);
    for header in &headers {
        raw_response.push_str(&format!("{}: {}\r\n", header.name, header.value));
    }
    raw_response.push_str("\r\n");
    raw_response.push_str(&body);

    debug!(
        "[HTTP] Check completed: {} - status={}, headers={}, time={:?}",
        url,
        status_code,
        headers.len(),
        elapsed
    );

    Ok(HttpHeaderCheckResult {
        url,
        status_code,
        status_text,
        response_time_ms: elapsed.as_millis() as u64,
        headers,
        security_analysis,
        content_length,
        raw_request,
        raw_response,
    })
}

/// 分析安全头
fn analyze_security_headers(headers: &[HttpHeader]) -> Vec<SecurityHeaderAnalysis> {
    let mut analysis = Vec::new();

    // 检查必需的安全头
    for &header_name in REQUIRED_SECURITY_HEADERS {
        let found = headers
            .iter()
            .find(|h| h.name.to_lowercase() == header_name);

        analysis.push(SecurityHeaderAnalysis {
            name: header_name.to_string(),
            present: found.is_some(),
            value: found.map(|h| h.value.clone()),
            status: if found.is_some() {
                "good".to_string()
            } else {
                "missing".to_string()
            },
            recommendation: if found.is_none() {
                Some(get_recommendation(header_name))
            } else {
                None
            },
        });
    }

    // 检查建议的安全头
    for &header_name in RECOMMENDED_SECURITY_HEADERS {
        let found = headers
            .iter()
            .find(|h| h.name.to_lowercase() == header_name);

        analysis.push(SecurityHeaderAnalysis {
            name: header_name.to_string(),
            present: found.is_some(),
            value: found.map(|h| h.value.clone()),
            status: if found.is_some() {
                "good".to_string()
            } else {
                "warning".to_string()
            },
            recommendation: if found.is_none() {
                Some(get_recommendation(header_name))
            } else {
                None
            },
        });
    }

    analysis
}

/// 获取安全头建议
fn get_recommendation(header_name: &str) -> String {
    match header_name {
        "strict-transport-security" => "Add HSTS header to enforce HTTPS connections".to_string(),
        "x-frame-options" => "Add to prevent clickjacking attacks".to_string(),
        "x-content-type-options" => "Set to 'nosniff' to prevent MIME type sniffing".to_string(),
        "content-security-policy" => "Add CSP header to prevent XSS attacks".to_string(),
        "referrer-policy" => "Set Referrer-Policy to control referrer information".to_string(),
        "permissions-policy" => "Set Permissions-Policy to restrict browser features".to_string(),
        "x-xss-protection" => "Add to enable browser XSS filter".to_string(),
        _ => "Consider adding this security header".to_string(),
    }
}
