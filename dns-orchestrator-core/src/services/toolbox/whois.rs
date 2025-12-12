//! WHOIS 查询模块

use regex::Regex;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::error::{CoreError, CoreResult};
use crate::types::WhoisResult;

/// WHOIS 查询
pub async fn whois_lookup(domain: &str, whois_servers: &str) -> CoreResult<WhoisResult> {
    let whois = WhoIs::from_string(whois_servers)
        .map_err(|e| CoreError::NetworkError(format!("初始化 WHOIS 客户端失败: {e}")))?;

    let options = WhoIsLookupOptions::from_string(domain)
        .map_err(|e| CoreError::ValidationError(format!("无效的域名: {e}")))?;

    let raw = whois
        .lookup_async(options)
        .await
        .map_err(|e| CoreError::NetworkError(format!("WHOIS 查询失败: {e}")))?;

    Ok(parse_whois_response(domain, &raw))
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
