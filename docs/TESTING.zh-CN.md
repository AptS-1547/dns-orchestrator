# Provider 集成测试指南

本指南说明如何运行 DNS Provider 库（`dns-orchestrator-provider`）的集成测试。

## 目录

- [概述](#概述)
- [环境变量](#环境变量)
- [运行测试](#运行测试)
- [测试结构](#测试结构)
- [编写新测试](#编写新测试)

## 概述

Provider 库包含集成测试，用于验证对 DNS 提供商的实际 API 调用。这些测试标记为 `#[ignore]`，在正常的 CI 构建中不会运行，因为它们需要真实的 API 凭证。

### 测试覆盖范围

每个 Provider 测试以下方法：

| 方法 | 说明 |
|------|------|
| `validate_credentials()` | 验证 API 凭证是否有效 |
| `list_domains()` | 列出账号下所有域名 |
| `get_domain()` | 获取指定域名的详情 |
| `list_records()` | 获取域名的 DNS 记录列表 |
| `create_record()` | 创建 DNS 记录 |
| `update_record()` | 更新 DNS 记录 |
| `delete_record()` | 删除 DNS 记录 |

## 环境变量

### 必需的环境变量

运行测试前需要设置以下环境变量：

```bash
# 测试域名（所有 Provider 都需要）
export TEST_DOMAIN=example.com

# Cloudflare
export CLOUDFLARE_API_TOKEN=your_api_token

# 阿里云 DNS
export ALIYUN_ACCESS_KEY_ID=your_access_key_id
export ALIYUN_ACCESS_KEY_SECRET=your_access_key_secret

# 腾讯云 DNSPod
export DNSPOD_SECRET_ID=your_secret_id
export DNSPOD_SECRET_KEY=your_secret_key

# 华为云 DNS
export HUAWEICLOUD_ACCESS_KEY_ID=your_access_key_id
export HUAWEICLOUD_SECRET_ACCESS_KEY=your_secret_access_key
```

### 获取 API 凭证

| Provider | 凭证获取文档 |
|----------|-------------|
| Cloudflare | [创建 API Token](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/) |
| 阿里云 | [创建 AccessKey](https://help.aliyun.com/document_detail/116401.html) |
| DNSPod | [创建 API 密钥](https://console.dnspod.cn/account/token/apikey) |
| 华为云 | [创建 AK/SK](https://support.huaweicloud.com/usermanual-ca/ca_01_0003.html) |

> **注意**：确保你的 API 凭证具有对测试域名的 DNS 记录管理权限。

## 运行测试

### 进入 Provider 目录

```bash
cd dns-orchestrator-provider
```

### 运行所有集成测试

```bash
cargo test -- --ignored --nocapture
```

### 运行指定 Provider 的测试

```bash
# Cloudflare
cargo test --test cloudflare_test -- --ignored --nocapture

# 阿里云
cargo test --test aliyun_test -- --ignored --nocapture

# DNSPod（使用 --test-threads=1 避免触发限流）
cargo test --test dnspod_test -- --ignored --nocapture --test-threads=1

# 华为云
cargo test --test huaweicloud_test -- --ignored --nocapture
```

> **注意**: DNSPod 有每秒 20 次请求的限制。使用 `--test-threads=1` 串行运行测试，避免 `QuotaExceeded` 错误。

### 运行单个测试

```bash
# 示例：只运行 Cloudflare 的 CRUD 测试
cargo test --test cloudflare_test test_cloudflare_crud_record -- --ignored --nocapture
```

### 命令参数说明

| 参数 | 说明 |
|------|------|
| `--ignored` | 运行标记为 `#[ignore]` 的测试 |
| `--nocapture` | 显示测试中的 `println!` 输出 |
| `--test <name>` | 运行指定的测试文件 |

## 测试结构

### 文件组织

```
dns-orchestrator-provider/
└── tests/
    ├── common/
    │   └── mod.rs          # 共享测试工具
    ├── cloudflare_test.rs  # Cloudflare 集成测试
    ├── aliyun_test.rs      # 阿里云集成测试
    ├── dnspod_test.rs      # DNSPod 集成测试
    └── huaweicloud_test.rs # 华为云集成测试
```

### 公共工具

`tests/common/mod.rs` 模块提供：

- **`skip_if_no_credentials!` 宏**：当环境变量缺失时跳过测试
- **`TestContext` 结构**：封装 Provider 实例和测试域名
- **`generate_test_record_name()`**：生成唯一的记录名（如 `_test-a1b2c3d4`）

### 测试记录命名规则

测试记录使用 `_test-` 前缀加上 UUID 片段：

```
_test-a1b2c3d4.example.com
```

这样可以方便识别和清理测试记录。

## 编写新测试

### 基本测试模板

```rust
#[tokio::test]
#[ignore]
async fn test_provider_feature() {
    skip_if_no_credentials!("PROVIDER_API_KEY", "TEST_DOMAIN");

    let ctx = TestContext::provider().expect("创建测试上下文失败");

    // 你的测试逻辑
    let result = ctx.provider.some_method().await;
    assert!(result.is_ok());

    println!("✓ 测试通过");
}
```

### CRUD 测试模式

测试记录操作的推荐模式：

```rust
#[tokio::test]
#[ignore]
async fn test_provider_crud_record() {
    skip_if_no_credentials!("PROVIDER_API_KEY", "TEST_DOMAIN");

    let mut ctx = TestContext::provider().expect("创建测试上下文失败");
    let domain_id = ctx.find_domain_id().await.expect("找不到域名");
    let record_name = common::generate_test_record_name();

    // 1. 创建
    let create_req = CreateDnsRecordRequest { ... };
    let created = ctx.provider.create_record(&create_req).await.unwrap();

    // 2. 读取（验证存在）
    let records = ctx.provider.list_records(&domain_id, &params).await.unwrap();
    assert!(records.items.iter().any(|r| r.id == created.id));

    // 3. 更新
    let update_req = UpdateDnsRecordRequest { ... };
    let updated = ctx.provider.update_record(&created.id, &update_req).await.unwrap();

    // 4. 删除
    ctx.provider.delete_record(&created.id, &domain_id).await.unwrap();

    // 5. 验证删除
    let records = ctx.provider.list_records(&domain_id, &params).await.unwrap();
    assert!(!records.items.iter().any(|r| r.id == created.id));
}
```

## 故障排除

### 测试跳过并提示 "Missing Environment Variable"

确保所有必需的环境变量已设置：

```bash
# 验证变量已设置
echo $CLOUDFLARE_API_TOKEN
echo $TEST_DOMAIN
```

### "Domain not found" 错误

- 确认 `TEST_DOMAIN` 与你 Provider 账号中的域名匹配
- 检查你的 API 凭证是否有权限访问该域名

### 记录创建失败

- 确保你的 API 凭证有写入权限
- 部分 Provider 有速率限制，稍等片刻后重试
- 检查是否已存在同名记录

### 清理残留的测试记录

如果测试失败并留下记录，可以手动清理或使用 `cleanup_all_test_records` 辅助函数：

```rust
ctx.cleanup_all_test_records(&domain_id).await;
```

这将删除所有名称中包含 `_test-` 的记录。
