//! DNSPod Provider 集成测试
//!
//! 运行方式:
//! ```bash
//! DNSPOD_SECRET_ID=xxx DNSPOD_SECRET_KEY=xxx TEST_DOMAIN=example.com \
//!     cargo test -p dns-orchestrator-provider --test dnspod_test -- --ignored --nocapture --test-threads=1
//! ```

mod common;

use common::{TestContext, TestRecordType, get_test_record_data};
use dns_orchestrator_provider::{
    CreateDnsRecordRequest, PaginationParams, RecordQueryParams, UpdateDnsRecordRequest,
};

// ============ 基础测试 ============

#[tokio::test]
#[ignore]
async fn test_dnspod_validate_credentials() {
    skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

    let ctx = TestContext::dnspod().expect("创建测试上下文失败");
    let result = ctx.provider.validate_credentials().await;

    assert!(
        result.is_ok(),
        "validate_credentials 调用失败: {:?}",
        result
    );
    assert!(result.unwrap(), "凭证应该有效");

    println!("✓ validate_credentials 测试通过");
}

#[tokio::test]
#[ignore]
async fn test_dnspod_list_domains() {
    skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

    let ctx = TestContext::dnspod().expect("创建测试上下文失败");
    let params = PaginationParams::default();

    let result = ctx.provider.list_domains(&params).await;
    assert!(result.is_ok(), "list_domains 调用失败: {:?}", result);

    let response = result.unwrap();
    assert!(!response.items.is_empty(), "域名列表不应为空");

    println!(
        "✓ list_domains 测试通过，共 {} 个域名",
        response.total_count
    );
}

#[tokio::test]
#[ignore]
async fn test_dnspod_get_domain() {
    skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

    let mut ctx = TestContext::dnspod().expect("创建测试上下文失败");
    let domain_id = ctx.find_domain_id().await.expect("找不到测试域名");

    let result = ctx.provider.get_domain(&domain_id).await;
    assert!(result.is_ok(), "get_domain 调用失败: {:?}", result);

    let domain = result.unwrap();
    assert_eq!(domain.name, ctx.domain, "域名名称不匹配");

    println!("✓ get_domain 测试通过: {}", domain.name);
}

#[tokio::test]
#[ignore]
async fn test_dnspod_list_records() {
    skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

    let mut ctx = TestContext::dnspod().expect("创建测试上下文失败");
    let domain_id = ctx.find_domain_id().await.expect("找不到测试域名");

    let params = RecordQueryParams::default();
    let result = ctx.provider.list_records(&domain_id, &params).await;
    assert!(result.is_ok(), "list_records 调用失败: {:?}", result);

    let response = result.unwrap();
    println!(
        "✓ list_records 测试通过，共 {} 条记录",
        response.total_count
    );
}

// ============ 清理测试 ============

/// 清理所有残留的测试记录（手动运行）
#[tokio::test]
#[ignore]
async fn test_dnspod_cleanup_test_records() {
    skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

    let mut ctx = TestContext::dnspod().expect("创建测试上下文失败");
    let domain_id = ctx.find_domain_id().await.expect("找不到测试域名");

    ctx.cleanup_all_test_records(&domain_id).await;
    println!("✓ 清理完成");
}

// ============ CRUD 测试宏 ============

macro_rules! crud_test {
    ($test_name:ident, $record_type:expr, $type_name:expr, $name_gen:expr) => {
        #[tokio::test]
        #[ignore]
        async fn $test_name() {
            skip_if_no_credentials!("DNSPOD_SECRET_ID", "DNSPOD_SECRET_KEY", "TEST_DOMAIN");

            let mut ctx = TestContext::dnspod().expect("创建测试上下文失败");
            let domain_id = ctx.find_domain_id().await.expect("找不到测试域名");

            let record_name = $name_gen();
            let (create_data, update_data) = get_test_record_data($record_type);

            println!("测试 {} 记录: {}", $type_name, record_name);

            // 0. 清理可能存在的同名记录（防止残留）
            let list_params = RecordQueryParams {
                page: 1,
                page_size: 100,
                keyword: None,
                record_type: None,
            };
            if let Ok(response) = ctx.provider.list_records(&domain_id, &list_params).await {
                for record in response.items {
                    if record.name.contains(&record_name) {
                        let _ = ctx.provider.delete_record(&record.id, &domain_id).await;
                        println!("  ⚠ 清理残留记录: {}", record.id);
                    }
                }
            }

            // 1. 创建记录
            let create_req = CreateDnsRecordRequest {
                domain_id: domain_id.clone(),
                name: record_name.clone(),
                ttl: 600,
                data: create_data,
                proxied: None,
            };

            let create_result = ctx.provider.create_record(&create_req).await;
            assert!(
                create_result.is_ok(),
                "create_record 失败: {:?}",
                create_result
            );

            let created_record = create_result.unwrap();
            let record_id = created_record.id.clone();
            println!("  ✓ 创建成功: id={}", record_id);

            // 2. 验证记录存在（不使用 keyword，遍历所有记录）
            let list_result = ctx.provider.list_records(&domain_id, &list_params).await;
            assert!(list_result.is_ok(), "list_records 失败: {:?}", list_result);

            let found = list_result.unwrap().items.iter().any(|r| r.id == record_id);
            assert!(found, "创建的记录应该能被找到");
            println!("  ✓ 验证存在");

            // 3. 更新记录
            let update_req = UpdateDnsRecordRequest {
                domain_id: domain_id.clone(),
                name: record_name.clone(),
                ttl: 1200, // DNSPod 要求 TTL >= 600
                data: update_data,
                proxied: None,
            };

            let update_result = ctx.provider.update_record(&record_id, &update_req).await;
            assert!(
                update_result.is_ok(),
                "update_record 失败: {:?}",
                update_result
            );

            let updated_record = update_result.unwrap();
            assert_eq!(updated_record.ttl, 1200, "TTL 应该被更新为 1200");
            println!("  ✓ 更新成功");

            // 4. 删除记录
            let delete_result = ctx.provider.delete_record(&record_id, &domain_id).await;
            assert!(
                delete_result.is_ok(),
                "delete_record 失败: {:?}",
                delete_result
            );
            println!("  ✓ 删除成功");

            // 5. 验证已删除
            let verify_result = ctx.provider.list_records(&domain_id, &list_params).await;
            if let Ok(response) = verify_result {
                let still_exists = response.items.iter().any(|r| r.id == record_id);
                assert!(!still_exists, "记录应该已被删除");
            }
            println!("  ✓ 验证删除");

            println!("✓ {} 记录 CRUD 测试通过", $type_name);
        }
    };
}

// ============ 各类型 CRUD 测试 ============

crud_test!(
    test_dnspod_crud_a_record,
    TestRecordType::A,
    "A",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_aaaa_record,
    TestRecordType::Aaaa,
    "AAAA",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_cname_record,
    TestRecordType::Cname,
    "CNAME",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_mx_record,
    TestRecordType::Mx,
    "MX",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_txt_record,
    TestRecordType::Txt,
    "TXT",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_caa_record,
    TestRecordType::Caa,
    "CAA",
    common::generate_test_record_name
);
crud_test!(
    test_dnspod_crud_srv_record,
    TestRecordType::Srv,
    "SRV",
    common::generate_srv_test_record_name
);
