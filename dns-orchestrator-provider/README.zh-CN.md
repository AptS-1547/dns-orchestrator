# dns-orchestrator-provider

用于统一管理多云 DNS 记录的 Provider 抽象库。

简体中文 | [English](./README.md)

## 功能特性

- **统一 Provider Trait** - 所有厂商通过同一个 `DnsProvider` 接口访问
- **完整 DNS 生命周期** - 支持常见记录类型的增删改查与批量操作
- **类型安全凭证** - `ProviderCredentials` 枚举避免凭证与厂商错配
- **统一错误模型** - 使用标准化 `ProviderError` 变体
- **瞬态失败自动重试** - 对网络/超时/限流错误使用指数退避重试
- **Feature Flag 控制编译** - 按需启用 Provider 与 TLS 后端

## 支持的 Provider

| Provider | Feature Flag | 认证方式 | 凭证字段 |
|----------|--------------|----------|----------|
| Cloudflare | `cloudflare` | Bearer Token | `api_token` |
| 阿里云 DNS | `aliyun` | ACS3-HMAC-SHA256 | `access_key_id`, `access_key_secret` |
| DNSPod | `dnspod` | TC3-HMAC-SHA256 | `secret_id`, `secret_key` |
| 华为云 DNS | `huaweicloud` | AK/SK Signing | `access_key_id`, `secret_access_key` |

## 快速开始

### 安装

启用全部 Provider（默认）：

```toml
[dependencies]
dns-orchestrator-provider = { version = "0.1", features = ["all-providers"] }
```

按需启用 Provider：

```toml
[dependencies]
dns-orchestrator-provider = { version = "0.1", default-features = false, features = ["cloudflare", "rustls"] }
```

### Feature Flags

Provider 相关：

- `all-providers`（默认）
- `cloudflare`
- `aliyun`
- `dnspod`
- `huaweicloud`

TLS 后端：

- `native-tls`
- `rustls`（默认）

## 使用示例

### 创建 Provider 并查询数据

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

### 创建记录

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

### 批量删除

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

## 错误处理

所有接口返回 `Result<T, ProviderError>`。

常见类别：

- 认证类：`InvalidCredentials`
- 资源冲突/不存在：`RecordExists`、`RecordNotFound`、`DomainNotFound`
- 参数/权限：`InvalidParameter`、`PermissionDenied`、`DomainLocked`
- 配额/限流：`QuotaExceeded`、`RateLimited`
- 基础设施：`NetworkError`、`Timeout`、`ParseError`、`SerializationError`

瞬态失败（`NetworkError`、`Timeout`、`RateLimited`）可重试。

## 架构

```text
Consumer (core/tauri/web)
  -> create_provider(credentials)
  -> Arc<dyn DnsProvider>
  -> 具体 provider 实现 (Cloudflare/Aliyun/DNSPod/Huawei)
  -> 共享 HTTP 工具 + 统一错误映射
```

详细文档：

- [架构说明](./docs/ARCHITECTURE.zh-CN.md)
- [测试指南](./docs/TESTING.zh-CN.md)

## 开发

```bash
# 在仓库根目录执行
cargo check -p dns-orchestrator-provider
cargo test -p dns-orchestrator-provider
```

## 许可证

MIT
