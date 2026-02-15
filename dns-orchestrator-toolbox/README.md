# dns-orchestrator-toolbox

Async network diagnostic toolkit for DNS and domain analysis.

[简体中文](./README.zh-CN.md) | English

## Features

- **WHOIS Lookup** - Query registration data with structured output
- **DNS Lookup** - Resolve A/AAAA/MX/TXT/NS/CNAME/SOA/SRV/CAA/PTR with optional custom nameserver (strongly-typed `DnsQueryType` enum)
- **DNS Propagation Check** - Compare answers across 13 global resolvers
- **DNSSEC Validation** - Inspect DNSKEY/DS/RRSIG and validation status
- **IP Geolocation** - Geolocate IP or domain-resolved addresses
- **SSL Certificate Inspection** - Read certificate details, validity, SAN, chain
- **HTTP Header Analysis** - Check response headers and security recommendations

## Core API

All capabilities are exposed as async associated functions on `ToolboxService` (stateless, no instance required).

| Method | Description |
|--------|-------------|
| `ToolboxService::whois_lookup` | WHOIS query |
| `ToolboxService::dns_lookup` | DNS record lookup |
| `ToolboxService::dns_propagation_check` | Multi-resolver propagation check |
| `ToolboxService::dnssec_check` | DNSSEC validation |
| `ToolboxService::ip_lookup` | IP geolocation |
| `ToolboxService::ssl_check` | SSL/TLS certificate inspection |
| `ToolboxService::http_header_check` | HTTP security header analysis |

## Quick Start

### Install

```toml
[dependencies]
dns-orchestrator-toolbox = "0.1"
```

### Usage

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

### HTTP Header Check

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

## Architecture

```
ToolboxService (stateless facade)
  -> whois / dns / dns_propagation / dnssec / ip / ssl / http_headers modules
  -> shared LazyLock singletons (DNS resolver, HTTP client)
  -> external network services (DNS resolvers, WHOIS servers, HTTPS endpoints, ipwho.is)
```

All methods are independent. DNS resolvers and HTTP clients are lazily initialised and reused across calls.

## DNS Propagation Servers

Propagation checks query 13 resolvers across multiple regions:

- North America: Google DNS, Cloudflare, Quad9, Level3
- Europe: Cloudflare (1.0.0.1), Quad9 (149.112.112.112), Google (8.8.4.4)
- Asia: Alibaba DNS, Tencent DNS, DNSPod
- Other: OpenDNS, AdGuard, Telstra

## Development

```bash
# From repository root
cargo check -p dns-orchestrator-toolbox
cargo test -p dns-orchestrator-toolbox
```

## License

MIT
