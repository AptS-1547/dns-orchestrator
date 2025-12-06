use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioResolver,
};
use regex::Regex;
use tauri::ipc::Channel;
use whois_rust::{WhoIs, WhoIsLookupOptions};

use crate::types::{ApiResponse, DnsLookupRecord, TracerouteHop, TracerouteProgress, WhoisResult};

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
) -> Result<ApiResponse<Vec<DnsLookupRecord>>, String> {
    let provider = TokioConnectionProvider::default();
    let resolver = TokioResolver::builder_with_config(ResolverConfig::default(), provider)
        .with_options(ResolverOpts::default())
        .build();

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
                            caa.value()
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
            // 查询所有常见类型
            let types = vec!["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SOA"];
            for t in types {
                if let Ok(ApiResponse {
                    data: Some(mut type_records),
                    ..
                }) = Box::pin(dns_lookup(domain.clone(), t.to_string())).await
                {
                    records.append(&mut type_records);
                }
            }
        }
        _ => {
            return Err(format!("不支持的记录类型: {}", record_type));
        }
    }

    Ok(ApiResponse::success(records))
}

/// Traceroute 查询
///
/// 使用系统 traceroute/tracert 命令，实时推送每跳结果
/// Android 平台不支持此功能
#[tauri::command]
pub async fn traceroute(
    target: String,
    on_progress: Channel<TracerouteProgress>,
) -> Result<ApiResponse<()>, String> {
    // Android 平台不支持
    #[cfg(target_os = "android")]
    {
        let _ = on_progress;
        return Err("当前平台不支持 traceroute 功能".to_string());
    }

    #[cfg(not(target_os = "android"))]
    {
        use std::process::Stdio;
        use tokio::io::{AsyncBufReadExt, BufReader};
        use tokio::process::Command;

        // 验证目标格式
        let target = target.trim();
        if target.is_empty() {
            return Err("目标地址不能为空".to_string());
        }

        // 根据平台选择命令
        let (cmd, args): (&str, Vec<&str>) = if cfg!(target_os = "windows") {
            ("tracert", vec!["-d", "-w", "3000", target]) // -d 不解析主机名, -w 超时3秒
        } else {
            ("traceroute", vec!["-n", "-w", "3", "-q", "1", target]) // -n 不解析, -w 超时3秒, -q 每跳1次探测
        };

        // 启动子进程
        let mut child = Command::new(cmd)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动 {} 失败: {}", cmd, e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法获取命令输出".to_string())?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        // 逐行解析输出
        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    if let Some(hop) = parse_traceroute_line(&line) {
                        let _ = on_progress.send(TracerouteProgress {
                            hop: Some(hop),
                            done: false,
                            error: None,
                        });
                    }
                }
                Err(_) => break, // 读取错误
            }
        }

        // 等待进程结束
        let status = child.wait().await.map_err(|e| e.to_string())?;

        // 发送完成事件
        let _ = on_progress.send(TracerouteProgress {
            hop: None,
            done: true,
            error: if status.success() {
                None
            } else {
                Some("traceroute 执行异常".to_string())
            },
        });

        Ok(ApiResponse::success(()))
    }
}

/// 解析 traceroute 输出行
///
/// macOS/Linux 格式:
///   1  192.168.1.1  1.234 ms  0.987 ms  1.123 ms
///   2  * * *
///   3  10.0.0.1  5.678 ms
///
/// Windows tracert 格式:
///   1    <1 ms    <1 ms    <1 ms  192.168.1.1
///   2     *        *        *     请求超时。
///   3    10 ms    12 ms    11 ms  10.0.0.1
#[cfg(not(target_os = "android"))]
fn parse_traceroute_line(line: &str) -> Option<TracerouteHop> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    // 跳过标题行（包含 "traceroute" 或 "Tracing"）
    if line.to_lowercase().contains("traceroute")
        || line.to_lowercase().contains("tracing")
        || line.to_lowercase().contains("over a maximum")
        || line.to_lowercase().contains("hops")
    {
        return None;
    }

    // 尝试解析跳数（行首的数字）
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    // 第一个部分应该是跳数
    let hop: u8 = parts.first()?.parse().ok()?;

    // 检查是否全是超时 (*)
    let rest = &parts[1..];
    let all_timeout = rest.iter().all(|p| *p == "*" || p.contains("超时") || p.contains("timed"));

    if all_timeout {
        return Some(TracerouteHop {
            hop,
            ip: None,
            hostname: None,
            rtt: vec![],
        });
    }

    // 提取 IP 地址
    let ip_re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").ok()?;
    let ip = ip_re
        .captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    // 提取 RTT 值
    let rtt_re = Regex::new(r"(\d+(?:\.\d+)?)\s*ms").ok()?;
    let rtt: Vec<f64> = rtt_re
        .captures_iter(line)
        .filter_map(|c| c.get(1))
        .filter_map(|m| m.as_str().parse().ok())
        .collect();

    // 处理 Windows 的 "<1 ms" 格式
    let lt_re = Regex::new(r"<(\d+)\s*ms").ok()?;
    let mut rtt = rtt;
    for cap in lt_re.captures_iter(line) {
        if let Some(m) = cap.get(1) {
            if let Ok(val) = m.as_str().parse::<f64>() {
                rtt.push(val);
            }
        }
    }

    Some(TracerouteHop {
        hop,
        ip,
        hostname: None, // 我们使用 -n 参数，不解析主机名
        rtt,
    })
}
