//! 共享测试工具和辅助函数

use std::env;
use std::sync::Arc;

use dns_orchestrator_provider::{
    CreateDnsRecordRequest, DnsProvider, DnsRecord, PaginationParams, ProviderCredentials,
    RecordData, RecordQueryParams, create_provider,
};

/// 跳过测试的宏（当环境变量缺失时）
#[macro_export]
macro_rules! skip_if_no_credentials {
    ($($var:expr),+) => {
        $(
            if std::env::var($var).is_err() {
                eprintln!("跳过测试: 缺少环境变量 {}", $var);
                return;
            }
        )+
    };
}

/// 生成唯一的测试记录名称
pub fn generate_test_record_name() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("_test-{}", &uuid.to_string()[..8])
}

/// 生成 SRV 记录专用的测试名称（格式：_service._tcp）
pub fn generate_srv_test_record_name() -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("_test-{}._tcp", &uuid.to_string()[..8])
}

/// 支持测试的记录类型
#[derive(Debug, Clone, Copy)]
pub enum TestRecordType {
    A,
    Aaaa,
    Cname,
    Mx,
    Txt,
    Srv,
    Caa,
}

/// 获取测试记录数据（创建和更新）
pub fn get_test_record_data(record_type: TestRecordType) -> (RecordData, RecordData) {
    match record_type {
        TestRecordType::A => (
            RecordData::A {
                address: "192.0.2.1".to_string(),
            },
            RecordData::A {
                address: "192.0.2.2".to_string(),
            },
        ),
        TestRecordType::Aaaa => (
            RecordData::AAAA {
                address: "2001:db8::1".to_string(),
            },
            RecordData::AAAA {
                address: "2001:db8::2".to_string(),
            },
        ),
        TestRecordType::Cname => (
            RecordData::CNAME {
                target: "target1.example.com".to_string(),
            },
            RecordData::CNAME {
                target: "target2.example.com".to_string(),
            },
        ),
        TestRecordType::Mx => (
            RecordData::MX {
                priority: 10,
                exchange: "mail1.example.com".to_string(),
            },
            RecordData::MX {
                priority: 20,
                exchange: "mail2.example.com".to_string(),
            },
        ),
        TestRecordType::Txt => (
            RecordData::TXT {
                text: "test-value-1".to_string(),
            },
            RecordData::TXT {
                text: "test-value-2".to_string(),
            },
        ),
        TestRecordType::Srv => (
            RecordData::SRV {
                priority: 0,
                weight: 5,
                port: 443,
                target: "srv1.example.com".to_string(),
            },
            RecordData::SRV {
                priority: 10,
                weight: 10,
                port: 8443,
                target: "srv2.example.com".to_string(),
            },
        ),
        TestRecordType::Caa => (
            RecordData::CAA {
                flags: 0,
                tag: "issue".to_string(),
                value: "letsencrypt.org".to_string(),
            },
            RecordData::CAA {
                flags: 0,
                tag: "issue".to_string(),
                value: "digicert.com".to_string(),
            },
        ),
    }
}

/// 测试上下文 - 封装 Provider 和测试域名
pub struct TestContext {
    pub provider: Arc<dyn DnsProvider>,
    pub domain: String,
    pub domain_id: Option<String>,
}

impl TestContext {
    /// 创建 Cloudflare 测试上下文
    pub fn cloudflare() -> Option<Self> {
        let api_token = env::var("CLOUDFLARE_API_TOKEN").ok()?;
        let domain = env::var("TEST_DOMAIN").ok()?;

        let credentials = ProviderCredentials::Cloudflare { api_token };
        let provider = create_provider(credentials).ok()?;

        Some(Self {
            provider,
            domain,
            domain_id: None,
        })
    }

    /// 创建 Aliyun 测试上下文
    pub fn aliyun() -> Option<Self> {
        let access_key_id = env::var("ALIYUN_ACCESS_KEY_ID").ok()?;
        let access_key_secret = env::var("ALIYUN_ACCESS_KEY_SECRET").ok()?;
        let domain = env::var("TEST_DOMAIN").ok()?;

        let credentials = ProviderCredentials::Aliyun {
            access_key_id,
            access_key_secret,
        };
        let provider = create_provider(credentials).ok()?;

        Some(Self {
            provider,
            domain,
            domain_id: None,
        })
    }

    /// 创建 DNSPod 测试上下文
    pub fn dnspod() -> Option<Self> {
        let secret_id = env::var("DNSPOD_SECRET_ID").ok()?;
        let secret_key = env::var("DNSPOD_SECRET_KEY").ok()?;
        let domain = env::var("TEST_DOMAIN").ok()?;

        let credentials = ProviderCredentials::Dnspod {
            secret_id,
            secret_key,
        };
        let provider = create_provider(credentials).ok()?;

        Some(Self {
            provider,
            domain,
            domain_id: None,
        })
    }

    /// 创建 Huaweicloud 测试上下文
    pub fn huaweicloud() -> Option<Self> {
        let access_key_id = env::var("HUAWEICLOUD_ACCESS_KEY_ID").ok()?;
        let secret_access_key = env::var("HUAWEICLOUD_SECRET_ACCESS_KEY").ok()?;
        let domain = env::var("TEST_DOMAIN").ok()?;

        let credentials = ProviderCredentials::Huaweicloud {
            access_key_id,
            secret_access_key,
        };
        let provider = create_provider(credentials).ok()?;

        Some(Self {
            provider,
            domain,
            domain_id: None,
        })
    }

    /// 查找测试域名的 domain_id
    pub async fn find_domain_id(&mut self) -> Option<String> {
        if self.domain_id.is_some() {
            return self.domain_id.clone();
        }

        let params = PaginationParams::default();
        let response = self.provider.list_domains(&params).await.ok()?;

        for domain in response.items {
            if domain.name == self.domain {
                self.domain_id = Some(domain.id.clone());
                return Some(domain.id);
            }
        }

        None
    }

    /// 创建测试记录并返回创建的记录
    pub async fn create_test_record(&self, domain_id: &str) -> Option<DnsRecord> {
        let record_name = generate_test_record_name();
        let request = CreateDnsRecordRequest {
            domain_id: domain_id.to_string(),
            name: record_name,
            ttl: 600,
            data: RecordData::TXT {
                text: "integration-test".to_string(),
            },
            proxied: None,
        };

        self.provider.create_record(&request).await.ok()
    }

    /// 清理测试记录
    pub async fn cleanup_record(&self, record_id: &str, domain_id: &str) {
        let _ = self.provider.delete_record(record_id, domain_id).await;
    }

    /// 查找并清理所有测试记录（以 _test- 开头的记录）
    pub async fn cleanup_all_test_records(&self, domain_id: &str) {
        let params = RecordQueryParams {
            page: 1,
            page_size: 100,
            keyword: Some("_test-".to_string()),
            record_type: None,
        };

        if let Ok(response) = self.provider.list_records(domain_id, &params).await {
            for record in response.items {
                if record.name.contains("_test-") {
                    let _ = self.provider.delete_record(&record.id, domain_id).await;
                }
            }
        }
    }
}
