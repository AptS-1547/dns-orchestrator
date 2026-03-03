# dns-orchestrator-toolbox

用于 DNS 与域名分析的异步网络诊断工具库。

简体中文 | [English](./README.md)

## 功能特性

- **WHOIS 查询** - 返回结构化域名注册信息
- **DNS 查询** - 支持 A/AAAA/MX/TXT/NS/CNAME/SOA/SRV/CAA/PTR，可选自定义 DNS 服务器（强类型 `DnsQueryType` 枚举）
- **DNS 传播检查** - 对比 13 个全球解析器的返回一致性
- **DNSSEC 校验** - 检查 DNSKEY/DS/RRSIG 与验证状态
- **IP 地理位置查询** - 支持 IP 或域名解析后地址定位
- **SSL 证书检查** - 查看证书有效期、SAN、证书链等信息
- **HTTP Header 分析** - 检查响应头并给出安全建议

## 核心 API

所有能力都通过 `ToolboxService` 的异步关联函数提供（无状态、无需实例化）。

| 方法 | 说明 |
|------|------|
| `ToolboxService::whois_lookup` | WHOIS 查询 |
| `ToolboxService::dns_lookup` | DNS 记录查询 |
| `ToolboxService::dns_propagation_check` | 多解析器传播检查 |
| `ToolboxService::dnssec_check` | DNSSEC 校验 |
| `ToolboxService::ip_lookup` | IP 地理位置查询 |
| `ToolboxService::ssl_check` | SSL/TLS 证书检查 |
| `ToolboxService::http_header_check` | HTTP 安全响应头分析 |

## 快速开始

### 安装

```toml
[dependencies]
dns-orchestrator-toolbox = "0.1"
```

### 使用示例

```rust,no_run
use dns_orchestrator_toolbox::{DnsQueryType, ToolboxResult, ToolboxService};

async fn run() -> ToolboxResult<()> {
    let whois = ToolboxService::whois_lookup("example.com").await?;
    println!("registrar: {:?}", whois.registrar);

    let dns = ToolboxService::dns_lookup("example.com", DnsQueryType::A, None).await?;
    println!("records: {}", dns.records.len());

    let propagation = ToolboxService::dns_propagation_check("example.com", DnsQueryType::A).await?;
    println!("consistency: {:.1}%", propagation.consistency_percentage);

    let dnssec = ToolboxService::dnssec_check("example.com", None).await?;
    println!("dnssec: {}", dnssec.validation_status);

    let ip = ToolboxService::ip_lookup("1.1.1.1").await?;
    println!("geo results: {}", ip.results.len());

    let ssl = ToolboxService::ssl_check("example.com", None).await?;
    println!("connection: {}", ssl.connection_status);

    Ok(())
}
```

### HTTP Header 分析

```rust,no_run
use dns_orchestrator_toolbox::{
    HttpHeaderCheckRequest, HttpMethod, ToolboxResult, ToolboxService,
};

async fn check_headers() -> ToolboxResult<()> {
    let request = HttpHeaderCheckRequest {
        url: "https://example.com".to_string(),
        method: HttpMethod::GET,
        custom_headers: vec![],
        body: None,
        content_type: None,
    };

    let result = ToolboxService::http_header_check(&request).await?;
    println!("status: {} {}", result.status_code, result.status_text);

    for item in &result.security_analysis {
        println!("{}: {:?}", item.name, item.status);
    }

    Ok(())
}
```

## 架构

```text
ToolboxService（无状态门面）
  -> whois / dns / dns_propagation / dnssec / ip / ssl / http_headers 模块
  -> 共享 LazyLock 单例（DNS 解析器、HTTP 客户端）
  -> 外部网络服务（DNS 解析器、WHOIS 服务器、HTTPS 端点、ipwho.is）
```

所有方法彼此独立。DNS 解析器和 HTTP 客户端懒初始化后跨调用复用。

## DNS 传播检查节点

传播检查会查询 13 个解析器，覆盖多个区域：

- 北美：Google DNS、Cloudflare、Quad9、Level3
- 欧洲：Cloudflare（1.0.0.1）、Quad9（149.112.112.112）、Google（8.8.4.4）
- 亚洲：阿里 DNS、腾讯 DNS、DNSPod
- 其他：OpenDNS、AdGuard、Telstra

## 开发

```bash
# 在仓库根目录执行
cargo check -p dns-orchestrator-toolbox
cargo test -p dns-orchestrator-toolbox
```

## 许可证

MIT
