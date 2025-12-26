# Provider Integration Testing Guide

This guide explains how to run integration tests for the DNS provider library (`dns-orchestrator-provider`).

## Table of Contents

- [Overview](#overview)
- [Environment Variables](#environment-variables)
- [Running Tests](#running-tests)
- [Test Structure](#test-structure)
- [Writing New Tests](#writing-new-tests)

## Overview

The provider library includes integration tests that verify actual API calls to DNS providers. These tests are marked with `#[ignore]` to prevent them from running during normal CI builds, as they require real API credentials.

### Test Coverage

Each provider has tests for:

| Method | Description |
|--------|-------------|
| `validate_credentials()` | Verify API credentials are valid |
| `list_domains()` | List all domains in the account |
| `get_domain()` | Get details of a specific domain |
| `list_records()` | List DNS records for a domain |
| `create_record()` | Create a new DNS record |
| `update_record()` | Update an existing DNS record |
| `delete_record()` | Delete a DNS record |

## Environment Variables

### Required Variables

Set these environment variables before running tests:

```bash
# Test domain (required for all providers)
export TEST_DOMAIN=example.com

# Cloudflare
export CLOUDFLARE_API_TOKEN=your_api_token

# Aliyun DNS
export ALIYUN_ACCESS_KEY_ID=your_access_key_id
export ALIYUN_ACCESS_KEY_SECRET=your_access_key_secret

# Tencent Cloud DNSPod
export DNSPOD_SECRET_ID=your_secret_id
export DNSPOD_SECRET_KEY=your_secret_key

# Huawei Cloud DNS
export HUAWEICLOUD_ACCESS_KEY_ID=your_access_key_id
export HUAWEICLOUD_SECRET_ACCESS_KEY=your_secret_access_key
```

### Getting API Credentials

| Provider | Credential Documentation |
|----------|-------------------------|
| Cloudflare | [Create API Token](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/) |
| Aliyun | [Create AccessKey](https://help.aliyun.com/document_detail/116401.html) |
| DNSPod | [Create API Key](https://console.dnspod.cn/account/token/apikey) |
| Huaweicloud | [Create AK/SK](https://support.huaweicloud.com/usermanual-ca/ca_01_0003.html) |

> **Note**: Ensure your API credentials have permissions for DNS record management on the test domain.

## Running Tests

### Navigate to Provider Directory

```bash
cd dns-orchestrator-provider
```

### Run All Integration Tests

```bash
cargo test -- --ignored --nocapture
```

### Run Tests for a Specific Provider

```bash
# Cloudflare
cargo test --test cloudflare_test -- --ignored --nocapture --test-threads=1

# Aliyun
cargo test --test aliyun_test -- --ignored --nocapture --test-threads=1

# DNSPod
cargo test --test dnspod_test -- --ignored --nocapture --test-threads=1

# Huaweicloud
cargo test --test huaweicloud_test -- --ignored --nocapture --test-threads=1
```

> **Note**: Always use `--test-threads=1` to run integration tests sequentially. Running tests in parallel may cause errors due to API rate limiting or concurrency issues (e.g., `RecordExists`, `QuotaExceeded`).

### Run a Specific Test

```bash
# Example: Run only the CRUD test for Cloudflare
cargo test --test cloudflare_test test_cloudflare_crud_record -- --ignored --nocapture
```

### Command Flags Explained

| Flag | Description |
|------|-------------|
| `--ignored` | Run tests marked with `#[ignore]` |
| `--nocapture` | Show `println!` output during tests |
| `--test <name>` | Run a specific test file |

## Test Structure

### File Organization

```
dns-orchestrator-provider/
└── tests/
    ├── common/
    │   └── mod.rs          # Shared test utilities
    ├── cloudflare_test.rs  # Cloudflare integration tests
    ├── aliyun_test.rs      # Aliyun integration tests
    ├── dnspod_test.rs      # DNSPod integration tests
    └── huaweicloud_test.rs # Huaweicloud integration tests
```

### Common Utilities

The `tests/common/mod.rs` module provides:

- **`skip_if_no_credentials!`** macro: Skips tests when environment variables are missing
- **`TestContext`** struct: Encapsulates provider instance and test domain
- **`generate_test_record_name()`**: Generates unique record names (e.g., `_test-a1b2c3d4`)

### Test Naming Convention

Test records are created with the prefix `_test-` followed by a UUID fragment:

```
_test-a1b2c3d4.example.com
```

This makes it easy to identify and clean up test records if needed.

## Writing New Tests

### Basic Test Template

```rust
#[tokio::test]
#[ignore]
async fn test_provider_feature() {
    skip_if_no_credentials!("PROVIDER_API_KEY", "TEST_DOMAIN");

    let ctx = TestContext::provider().expect("Failed to create test context");

    // Your test logic here
    let result = ctx.provider.some_method().await;
    assert!(result.is_ok());

    println!("✓ Test passed");
}
```

### CRUD Test Pattern

The recommended pattern for testing record operations:

```rust
#[tokio::test]
#[ignore]
async fn test_provider_crud_record() {
    skip_if_no_credentials!("PROVIDER_API_KEY", "TEST_DOMAIN");

    let mut ctx = TestContext::provider().expect("Failed to create test context");
    let domain_id = ctx.find_domain_id().await.expect("Domain not found");
    let record_name = common::generate_test_record_name();

    // 1. Create
    let create_req = CreateDnsRecordRequest { ... };
    let created = ctx.provider.create_record(&create_req).await.unwrap();

    // 2. Read (verify existence)
    let records = ctx.provider.list_records(&domain_id, &params).await.unwrap();
    assert!(records.items.iter().any(|r| r.id == created.id));

    // 3. Update
    let update_req = UpdateDnsRecordRequest { ... };
    let updated = ctx.provider.update_record(&created.id, &update_req).await.unwrap();

    // 4. Delete
    ctx.provider.delete_record(&created.id, &domain_id).await.unwrap();

    // 5. Verify deletion
    let records = ctx.provider.list_records(&domain_id, &params).await.unwrap();
    assert!(!records.items.iter().any(|r| r.id == created.id));
}
```

## Troubleshooting

### Tests Skip with "Missing Environment Variable"

Ensure all required environment variables are set:

```bash
# Verify variables are set
echo $CLOUDFLARE_API_TOKEN
echo $TEST_DOMAIN
```

### "Domain not found" Error

- Verify `TEST_DOMAIN` matches a domain in your provider account
- Check that your API credentials have access to that domain

### Record Creation Fails

- Ensure your API credentials have write permissions
- Some providers have rate limits; wait a moment and retry
- Check if a record with the same name already exists

### Cleanup Leftover Test Records

If tests fail and leave behind records, you can manually clean them up or use the `cleanup_all_test_records` helper:

```rust
ctx.cleanup_all_test_records(&domain_id).await;
```

This will delete all records with names containing `_test-`.
