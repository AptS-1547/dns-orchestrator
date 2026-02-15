# dns-orchestrator-provider

Unified DNS provider abstraction library for managing DNS records across multiple cloud platforms.

[简体中文](./README.zh-CN.md) | English

## Features

- **Unified Provider Trait** - Single `DnsProvider` interface for all supported providers
- **Full DNS Record Lifecycle** - Create, read, update, delete, and batch operations for common record types
- **Type-Safe Credentials** - `ProviderCredentials` enum prevents provider/credential mismatch
- **Consistent Error Model** - Standardized `ProviderError` variants across providers
- **Retry on Transient Failures** - Automatic retry with exponential backoff for network/timeout/rate-limit errors
- **Feature-Flag Driven** - Enable only required providers and TLS backend

## Supported Providers

| Provider | Feature Flag | Auth Method | Credential Fields |
|----------|--------------|-------------|-------------------|
| Cloudflare | `cloudflare` | Bearer Token | `api_token` |
| Aliyun DNS | `aliyun` | ACS3-HMAC-SHA256 | `access_key_id`, `access_key_secret` |
| DNSPod | `dnspod` | TC3-HMAC-SHA256 | `secret_id`, `secret_key` |
| Huawei Cloud DNS | `huaweicloud` | AK/SK Signing | `access_key_id`, `secret_access_key` |

## Quick Start

### Install

Enable all providers (default):

```toml
[dependencies]
dns-orchestrator-provider = { version = "0.1", features = ["all-providers"] }
```

Enable only selected providers:

```toml
[dependencies]
dns-orchestrator-provider = { version = "0.1", default-features = false, features = ["cloudflare", "rustls"] }
```

### Feature Flags

Provider flags:

- `all-providers` (default)
- `cloudflare`
- `aliyun`
- `dnspod`
- `huaweicloud`

TLS backend flags:

- `native-tls`
- `rustls` (default)

## Usage

### Create Provider and Query Data

```rust,no_run
use dns_orchestrator_provider::{create_provider, PaginationParams, ProviderCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = ProviderCredentials::Cloudflare {
        api_token: "your-api-token".to_string(),
    };

    let provider = create_provider(credentials)?;

    provider.validate_credentials().await?;

    let domains = provider.list_domains(&PaginationParams::default()).await?;
    for domain in &domains.items {
        println!("{} ({:?})", domain.name, domain.status);
    }

    Ok(())
}
```

### Create Record

```rust,no_run
# use dns_orchestrator_provider::{create_provider, CreateDnsRecordRequest, ProviderCredentials, RecordData};
# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
# let provider = create_provider(ProviderCredentials::Cloudflare { api_token: "token".to_string() })?;
let request = CreateDnsRecordRequest {
    domain_id: "example.com".to_string(),
    name: "www".to_string(),
    ttl: 600,
    data: RecordData::A { address: "1.2.3.4".to_string() },
    proxied: None,
};

let record = provider.create_record(&request).await?;
println!("created: {}", record.id);
# Ok(())
# }
```

### Batch Delete

```rust,no_run
# use dns_orchestrator_provider::{create_provider, ProviderCredentials};
# async fn demo() -> Result<(), Box<dyn std::error::Error>> {
# let provider = create_provider(ProviderCredentials::Cloudflare { api_token: "token".to_string() })?;
let result = provider
    .batch_delete_records(
        "example.com",
        &["record-1".to_string(), "record-2".to_string()],
    )
    .await?;

println!("success={}, failed={}", result.success_count, result.failed_count);
for failure in &result.failures {
    eprintln!("{}: {}", failure.record_id, failure.reason);
}
# Ok(())
# }
```

## Error Handling

All operations return `Result<T, ProviderError>`.

Common categories:

- Authentication: `InvalidCredentials`
- Resource conflicts/missing: `RecordExists`, `RecordNotFound`, `DomainNotFound`
- Validation/permission: `InvalidParameter`, `PermissionDenied`, `DomainLocked`
- Capacity/limits: `QuotaExceeded`, `RateLimited`
- Infrastructure: `NetworkError`, `Timeout`, `ParseError`, `SerializationError`

Transient failures (`NetworkError`, `Timeout`, `RateLimited`) are retryable.

## Architecture

```
Consumer (core/tauri/web)
  -> create_provider(credentials)
  -> Arc<dyn DnsProvider>
  -> provider-specific implementation (Cloudflare/Aliyun/DNSPod/Huawei)
  -> shared HTTP utility + unified error mapping
```

Detailed docs:

- [Architecture](./docs/ARCHITECTURE.md)
- [Testing Guide](./docs/TESTING.md)

## Development

```bash
# From repository root
cargo check -p dns-orchestrator-provider
cargo test -p dns-orchestrator-provider
```

## License

MIT
