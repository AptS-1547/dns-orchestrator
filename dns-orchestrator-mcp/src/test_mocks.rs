use super::*;

use std::collections::HashMap;

use dns_orchestrator_core::error::{CoreError, CoreResult};
use dns_orchestrator_core::traits::{
    AccountRepository, CredentialStore, CredentialsMap, DomainMetadataRepository,
    InMemoryProviderRegistry, ProviderRegistry,
};
use dns_orchestrator_core::types::{
    Account, AccountStatus, DnsRecord, DomainStatus, PaginatedResponse, PaginationParams,
    ProviderCredentials, ProviderDomain, ProviderType, RecordData, RecordQueryParams,
};
use dns_orchestrator_provider::{
    CreateDnsRecordRequest, DnsProvider, ProviderError, ProviderFeatures, ProviderLimits,
    ProviderMetadata, UpdateDnsRecordRequest,
};
use tokio::sync::{Mutex, RwLock};

/// Test-only no-op domain metadata repository.
pub struct NoOpDomainMetadataRepository;

#[async_trait]
impl DomainMetadataRepository for NoOpDomainMetadataRepository {
    async fn find_by_key(
        &self,
        _key: &dns_orchestrator_core::types::DomainMetadataKey,
    ) -> CoreResult<Option<dns_orchestrator_core::types::DomainMetadata>> {
        Ok(None)
    }
    async fn find_by_keys(
        &self,
        _keys: &[dns_orchestrator_core::types::DomainMetadataKey],
    ) -> CoreResult<
        HashMap<
            dns_orchestrator_core::types::DomainMetadataKey,
            dns_orchestrator_core::types::DomainMetadata,
        >,
    > {
        Ok(HashMap::new())
    }
    async fn save(
        &self,
        _key: &dns_orchestrator_core::types::DomainMetadataKey,
        _metadata: &dns_orchestrator_core::types::DomainMetadata,
    ) -> CoreResult<()> {
        Ok(())
    }
    async fn batch_save(
        &self,
        _entries: &[(
            dns_orchestrator_core::types::DomainMetadataKey,
            dns_orchestrator_core::types::DomainMetadata,
        )],
    ) -> CoreResult<()> {
        Ok(())
    }
    async fn update(
        &self,
        _key: &dns_orchestrator_core::types::DomainMetadataKey,
        _update: &dns_orchestrator_core::types::DomainMetadataUpdate,
    ) -> CoreResult<()> {
        Ok(())
    }
    async fn delete(
        &self,
        _key: &dns_orchestrator_core::types::DomainMetadataKey,
    ) -> CoreResult<()> {
        Ok(())
    }
    async fn delete_by_account(&self, _account_id: &str) -> CoreResult<()> {
        Ok(())
    }
    async fn find_favorites_by_account(
        &self,
        _account_id: &str,
    ) -> CoreResult<Vec<dns_orchestrator_core::types::DomainMetadataKey>> {
        Ok(Vec::new())
    }
    async fn find_by_tag(
        &self,
        _tag: &str,
    ) -> CoreResult<Vec<dns_orchestrator_core::types::DomainMetadataKey>> {
        Ok(Vec::new())
    }
    async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        Ok(Vec::new())
    }
}

pub struct TestCredentialStore;

#[async_trait]
impl CredentialStore for TestCredentialStore {
    async fn load_all(&self) -> CoreResult<CredentialsMap> {
        Ok(HashMap::new())
    }

    async fn save_all(&self, _credentials: &CredentialsMap) -> CoreResult<()> {
        Ok(())
    }

    async fn get(&self, _account_id: &str) -> CoreResult<Option<ProviderCredentials>> {
        Ok(None)
    }

    async fn set(&self, _account_id: &str, _credentials: &ProviderCredentials) -> CoreResult<()> {
        Ok(())
    }

    async fn remove(&self, _account_id: &str) -> CoreResult<()> {
        Ok(())
    }

    async fn load_raw_json(&self) -> CoreResult<String> {
        Ok("{}".to_string())
    }

    async fn save_raw_json(&self, _json: &str) -> CoreResult<()> {
        Ok(())
    }
}

pub struct TestAccountRepository {
    accounts: RwLock<Vec<Account>>,
    fail_find_all: bool,
}

impl TestAccountRepository {
    pub fn new(accounts: Vec<Account>) -> Self {
        Self {
            accounts: RwLock::new(accounts),
            fail_find_all: false,
        }
    }

    pub fn failing_find_all() -> Self {
        Self {
            accounts: RwLock::new(Vec::new()),
            fail_find_all: true,
        }
    }
}

#[async_trait]
impl AccountRepository for TestAccountRepository {
    async fn find_all(&self) -> CoreResult<Vec<Account>> {
        if self.fail_find_all {
            return Err(CoreError::StorageError("mock find_all failure".to_string()));
        }
        Ok(self.accounts.read().await.clone())
    }

    async fn find_by_id(&self, id: &str) -> CoreResult<Option<Account>> {
        Ok(self
            .accounts
            .read()
            .await
            .iter()
            .find(|account| account.id == id)
            .cloned())
    }

    async fn save(&self, account: &Account) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(existing) = accounts.iter_mut().find(|item| item.id == account.id) {
            *existing = account.clone();
        } else {
            accounts.push(account.clone());
        }
        Ok(())
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        accounts.retain(|account| account.id != id);
        Ok(())
    }

    async fn save_all(&self, accounts: &[Account]) -> CoreResult<()> {
        *self.accounts.write().await = accounts.to_vec();
        Ok(())
    }

    async fn update_status(
        &self,
        id: &str,
        status: AccountStatus,
        error: Option<String>,
    ) -> CoreResult<()> {
        let mut accounts = self.accounts.write().await;
        let Some(account) = accounts.iter_mut().find(|item| item.id == id) else {
            return Err(CoreError::AccountNotFound(id.to_string()));
        };

        account.status = Some(status);
        account.error = error;
        account.updated_at = chrono::Utc::now();
        Ok(())
    }
}

#[derive(Default)]
pub struct MockDnsProvider {
    domain_params: Mutex<Option<PaginationParams>>,
    records_domain_id: Mutex<Option<String>>,
    record_params: Mutex<Option<RecordQueryParams>>,
}

impl MockDnsProvider {
    pub async fn domain_params(&self) -> Option<PaginationParams> {
        self.domain_params.lock().await.clone()
    }

    pub async fn records_domain_id(&self) -> Option<String> {
        self.records_domain_id.lock().await.clone()
    }

    pub async fn record_params(&self) -> Option<RecordQueryParams> {
        self.record_params.lock().await.clone()
    }
}

#[async_trait]
impl DnsProvider for MockDnsProvider {
    fn id(&self) -> &'static str {
        "mock"
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            id: ProviderType::Cloudflare,
            name: "Mock Provider".to_string(),
            description: "Test provider".to_string(),
            required_fields: Vec::new(),
            features: ProviderFeatures::default(),
            limits: ProviderLimits {
                max_page_size_domains: 100,
                max_page_size_records: 100,
            },
        }
    }

    async fn validate_credentials(&self) -> dns_orchestrator_provider::Result<bool> {
        Ok(true)
    }

    async fn list_domains(
        &self,
        params: &PaginationParams,
    ) -> dns_orchestrator_provider::Result<PaginatedResponse<ProviderDomain>> {
        *self.domain_params.lock().await = Some(params.clone());

        Ok(PaginatedResponse::new(
            vec![ProviderDomain {
                id: "dom-1".to_string(),
                name: "example.com".to_string(),
                provider: ProviderType::Cloudflare,
                status: DomainStatus::Active,
                record_count: Some(1),
            }],
            params.page,
            params.page_size,
            1,
        ))
    }

    async fn get_domain(
        &self,
        domain_id: &str,
    ) -> dns_orchestrator_provider::Result<ProviderDomain> {
        Ok(ProviderDomain {
            id: domain_id.to_string(),
            name: "example.com".to_string(),
            provider: ProviderType::Cloudflare,
            status: DomainStatus::Active,
            record_count: Some(1),
        })
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> dns_orchestrator_provider::Result<PaginatedResponse<DnsRecord>> {
        *self.records_domain_id.lock().await = Some(domain_id.to_string());
        *self.record_params.lock().await = Some(params.clone());

        Ok(PaginatedResponse::new(
            vec![DnsRecord {
                id: "rec-1".to_string(),
                domain_id: domain_id.to_string(),
                name: "www".to_string(),
                ttl: 300,
                data: RecordData::A {
                    address: "1.1.1.1".to_string(),
                },
                proxied: None,
                created_at: None,
                updated_at: None,
            }],
            params.page,
            params.page_size,
            1,
        ))
    }

    async fn create_record(
        &self,
        _req: &CreateDnsRecordRequest,
    ) -> dns_orchestrator_provider::Result<DnsRecord> {
        Err(ProviderError::Unknown {
            provider: "mock".to_string(),
            raw_code: None,
            raw_message: "not implemented".to_string(),
        })
    }

    async fn update_record(
        &self,
        _record_id: &str,
        _req: &UpdateDnsRecordRequest,
    ) -> dns_orchestrator_provider::Result<DnsRecord> {
        Err(ProviderError::Unknown {
            provider: "mock".to_string(),
            raw_code: None,
            raw_message: "not implemented".to_string(),
        })
    }

    async fn delete_record(
        &self,
        _record_id: &str,
        _domain_id: &str,
    ) -> dns_orchestrator_provider::Result<()> {
        Err(ProviderError::Unknown {
            provider: "mock".to_string(),
            raw_code: None,
            raw_message: "not implemented".to_string(),
        })
    }
}

#[derive(Default)]
pub struct MockToolboxGateway {
    dns_lookup_calls: Mutex<Vec<(String, DnsQueryType, Option<String>)>>,
    dns_lookup_delay: Mutex<Option<Duration>>,
    dns_lookup_error: Mutex<Option<String>>,
    whois_calls: Mutex<Vec<String>>,
    ip_calls: Mutex<Vec<String>>,
    propagation_calls: Mutex<Vec<(String, DnsQueryType)>>,
    dnssec_calls: Mutex<Vec<(String, Option<String>)>>,
}

impl MockToolboxGateway {
    pub async fn set_dns_lookup_delay(&self, delay: Option<Duration>) {
        *self.dns_lookup_delay.lock().await = delay;
    }

    pub async fn set_dns_lookup_error(&self, error: Option<String>) {
        *self.dns_lookup_error.lock().await = error;
    }

    pub async fn dns_lookup_calls(&self) -> Vec<(String, DnsQueryType, Option<String>)> {
        self.dns_lookup_calls.lock().await.clone()
    }

    pub async fn whois_calls(&self) -> Vec<String> {
        self.whois_calls.lock().await.clone()
    }

    pub async fn ip_calls(&self) -> Vec<String> {
        self.ip_calls.lock().await.clone()
    }

    pub async fn propagation_calls(&self) -> Vec<(String, DnsQueryType)> {
        self.propagation_calls.lock().await.clone()
    }

    pub async fn dnssec_calls(&self) -> Vec<(String, Option<String>)> {
        self.dnssec_calls.lock().await.clone()
    }
}

#[async_trait]
impl ToolboxGateway for MockToolboxGateway {
    async fn dns_lookup(
        &self,
        domain: &str,
        record_type: DnsQueryType,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnsLookupResult> {
        self.dns_lookup_calls.lock().await.push((
            domain.to_string(),
            record_type,
            nameserver.map(std::string::ToString::to_string),
        ));

        if let Some(delay) = *self.dns_lookup_delay.lock().await {
            tokio::time::sleep(delay).await;
        }

        if let Some(message) = self.dns_lookup_error.lock().await.clone() {
            return Err(ToolboxError::NetworkError(message));
        }

        Ok(DnsLookupResult {
            nameserver: nameserver.unwrap_or("system").to_string(),
            records: vec![dns_record()],
        })
    }

    async fn whois_lookup(&self, domain: &str) -> ToolboxResult<WhoisResult> {
        self.whois_calls.lock().await.push(domain.to_string());
        Ok(WhoisResult {
            domain: domain.to_string(),
            registrar: Some("Mock Registrar".to_string()),
            creation_date: None,
            expiration_date: None,
            updated_date: None,
            name_servers: vec!["ns1.example.com".to_string()],
            status: vec!["active".to_string()],
            raw: "mock-raw".to_string(),
        })
    }

    async fn ip_lookup(&self, query: &str) -> ToolboxResult<IpLookupResult> {
        self.ip_calls.lock().await.push(query.to_string());
        Ok(IpLookupResult {
            query: query.to_string(),
            is_domain: false,
            results: vec![dns_orchestrator_toolbox::IpGeoInfo {
                ip: "1.1.1.1".to_string(),
                ip_version: "IPv4".to_string(),
                country: Some("US".to_string()),
                country_code: Some("US".to_string()),
                region: None,
                city: None,
                latitude: None,
                longitude: None,
                timezone: None,
                isp: None,
                org: None,
                asn: None,
                as_name: None,
            }],
        })
    }

    async fn dns_propagation_check(
        &self,
        domain: &str,
        record_type: DnsQueryType,
    ) -> ToolboxResult<DnsPropagationResult> {
        self.propagation_calls
            .lock()
            .await
            .push((domain.to_string(), record_type));
        Ok(DnsPropagationResult {
            domain: domain.to_string(),
            record_type,
            results: vec![dns_orchestrator_toolbox::DnsPropagationServerResult {
                server: dns_orchestrator_toolbox::DnsPropagationServer {
                    name: "Mock DNS".to_string(),
                    ip: "1.1.1.1".to_string(),
                    region: "NA".to_string(),
                    country_code: "US".to_string(),
                },
                status: dns_orchestrator_toolbox::PropagationStatus::Success,
                records: vec![dns_record()],
                error: None,
                response_time_ms: 10,
            }],
            total_time_ms: 10,
            consistency_percentage: 100.0,
            unique_values: vec!["1.1.1.1".to_string()],
        })
    }

    async fn dnssec_check(
        &self,
        domain: &str,
        nameserver: Option<&str>,
    ) -> ToolboxResult<DnssecResult> {
        self.dnssec_calls.lock().await.push((
            domain.to_string(),
            nameserver.map(std::string::ToString::to_string),
        ));
        Ok(DnssecResult {
            domain: domain.to_string(),
            dnssec_enabled: false,
            dnskey_records: Vec::new(),
            ds_records: Vec::new(),
            rrsig_records: Vec::new(),
            validation_status: dns_orchestrator_toolbox::DnssecValidationStatus::Insecure,
            nameserver: nameserver.unwrap_or("system").to_string(),
            response_time_ms: 10,
            error: None,
        })
    }
}

pub fn dns_record() -> dns_orchestrator_toolbox::DnsLookupRecord {
    dns_orchestrator_toolbox::DnsLookupRecord {
        record_type: "A".to_string(),
        name: "example.com".to_string(),
        value: "1.1.1.1".to_string(),
        ttl: 300,
        priority: None,
    }
}

pub fn test_account(account_id: &str) -> Account {
    Account {
        id: account_id.to_string(),
        name: "Test Account".to_string(),
        provider: ProviderType::Cloudflare,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: Some(AccountStatus::Active),
        error: None,
    }
}

pub(super) async fn build_server(
    account_repository: Arc<dyn AccountRepository>,
    provider: Option<Arc<MockDnsProvider>>,
    toolbox: Arc<dyn ToolboxGateway>,
    timeouts: ToolTimeouts,
) -> DnsOrchestratorMcp {
    let credential_store: Arc<dyn CredentialStore> = Arc::new(TestCredentialStore);
    let provider_registry = Arc::new(InMemoryProviderRegistry::new());

    if let Some(mock_provider) = provider {
        provider_registry
            .register("acc-1".to_string(), mock_provider)
            .await;
    }

    let domain_metadata_repository: Arc<dyn DomainMetadataRepository> =
        Arc::new(NoOpDomainMetadataRepository);

    let ctx = Arc::new(ServiceContext::new(
        credential_store,
        account_repository,
        provider_registry,
        Arc::clone(&domain_metadata_repository),
    ));

    let account_service = Arc::new(AccountService::new(Arc::clone(&ctx)));
    let domain_metadata_service = Arc::new(DomainMetadataService::new(domain_metadata_repository));

    DnsOrchestratorMcp::with_toolbox_and_timeouts(
        &ctx,
        account_service,
        domain_metadata_service,
        toolbox,
        timeouts,
    )
}
