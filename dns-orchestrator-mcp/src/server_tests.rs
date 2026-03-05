use super::test_mocks::*;
use super::*;

use dns_orchestrator_core::traits::AccountRepository;
use dns_orchestrator_core::types::DnsRecordType as ProviderDnsRecordType;
use dns_orchestrator_toolbox::DnsQueryType;

use crate::schemas::{
    DnsLookupParams, DnsPropagationCheckParams, DnssecCheckParams, IpLookupParams,
    ListAccountsParams, ListDomainsParams, ListRecordsParams, WhoisLookupParams,
};

#[test]
fn sanitize_internal_error_hides_error_details() {
    let error = sanitize_internal_error("sensitive: token=123", "List accounts");
    let message = error.to_string();
    assert!(message.contains("List accounts failed"));
    assert!(!message.contains("token=123"));
}

#[tokio::test]
async fn list_accounts_returns_success() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(account_repository, None, toolbox, ToolTimeouts::default()).await;

    let result = server
        .list_accounts(Parameters(ListAccountsParams {}))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn list_accounts_error_is_sanitized() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::failing_find_all());
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(account_repository, None, toolbox, ToolTimeouts::default()).await;

    let error = server
        .list_accounts(Parameters(ListAccountsParams {}))
        .await
        .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("List accounts failed"));
    assert!(!message.contains("mock find_all failure"));
}

#[tokio::test]
async fn list_domains_clamps_page_size() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let provider = Arc::new(MockDnsProvider::default());
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(
        account_repository,
        Some(Arc::clone(&provider)),
        toolbox,
        ToolTimeouts::default(),
    )
    .await;

    let result = server
        .list_domains(Parameters(ListDomainsParams {
            account_id: "acc-1".to_string(),
            page: Some(2),
            page_size: Some(999),
        }))
        .await;

    assert!(result.is_ok());

    let params = provider.domain_params().await.unwrap();
    assert_eq!(params.page, 2);
    assert_eq!(params.page_size, 100);
}

#[tokio::test]
async fn list_records_maps_type_and_clamps_page_size() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let provider = Arc::new(MockDnsProvider::default());
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(
        account_repository,
        Some(Arc::clone(&provider)),
        toolbox,
        ToolTimeouts::default(),
    )
    .await;

    let result = server
        .list_records(Parameters(ListRecordsParams {
            account_id: "acc-1".to_string(),
            domain_id: "dom-1".to_string(),
            page: Some(1),
            page_size: Some(500),
            keyword: Some("www".to_string()),
            record_type: Some("mx".to_string()),
        }))
        .await;

    assert!(result.is_ok());
    assert_eq!(
        provider.records_domain_id().await,
        Some("dom-1".to_string())
    );

    let params = provider.record_params().await.unwrap();
    assert_eq!(params.page_size, 100);
    assert_eq!(params.keyword, Some("www".to_string()));
    assert_eq!(params.record_type, Some(ProviderDnsRecordType::Mx));
}

#[tokio::test]
async fn list_records_unknown_type_becomes_none() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let provider = Arc::new(MockDnsProvider::default());
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(
        account_repository,
        Some(Arc::clone(&provider)),
        toolbox,
        ToolTimeouts::default(),
    )
    .await;

    let result = server
        .list_records(Parameters(ListRecordsParams {
            account_id: "acc-1".to_string(),
            domain_id: "dom-1".to_string(),
            page: Some(1),
            page_size: Some(20),
            keyword: None,
            record_type: Some("UNKNOWN".to_string()),
        }))
        .await;

    assert!(result.is_ok());

    let params = provider.record_params().await.unwrap();
    assert!(params.record_type.is_none());
}

#[tokio::test]
async fn dns_lookup_uses_toolbox_arguments() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(
        account_repository,
        None,
        Arc::clone(&toolbox) as Arc<dyn ToolboxGateway>,
        ToolTimeouts::default(),
    )
    .await;

    let result = server
        .dns_lookup(Parameters(DnsLookupParams {
            domain: "example.com".to_string(),
            record_type: DnsQueryType::A,
            nameserver: Some("8.8.8.8".to_string()),
        }))
        .await;

    assert!(result.is_ok());

    let calls = toolbox.dns_lookup_calls().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "example.com");
    assert_eq!(calls[0].1, DnsQueryType::A);
    assert_eq!(calls[0].2, Some("8.8.8.8".to_string()));
}

#[tokio::test]
async fn dns_lookup_returns_toolbox_error_message() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());
    toolbox
        .set_dns_lookup_error(Some("mock dns lookup failed".to_string()))
        .await;

    let server = build_server(
        account_repository,
        None,
        Arc::clone(&toolbox) as Arc<dyn ToolboxGateway>,
        ToolTimeouts::default(),
    )
    .await;

    let error = server
        .dns_lookup(Parameters(DnsLookupParams {
            domain: "example.com".to_string(),
            record_type: DnsQueryType::A,
            nameserver: None,
        }))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("mock dns lookup failed"));
}

#[tokio::test]
async fn dns_lookup_timeout_returns_timeout_error() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());
    toolbox
        .set_dns_lookup_delay(Some(Duration::from_millis(50)))
        .await;

    let timeouts = ToolTimeouts {
        dns_lookup: Duration::from_millis(5),
        ..ToolTimeouts::default()
    };

    let server = build_server(
        account_repository,
        None,
        Arc::clone(&toolbox) as Arc<dyn ToolboxGateway>,
        timeouts,
    )
    .await;

    let error = server
        .dns_lookup(Parameters(DnsLookupParams {
            domain: "example.com".to_string(),
            record_type: DnsQueryType::A,
            nameserver: None,
        }))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("DNS lookup timeout"));
}

#[tokio::test]
async fn toolbox_tools_delegate_to_gateway() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());

    let server = build_server(
        account_repository,
        None,
        Arc::clone(&toolbox) as Arc<dyn ToolboxGateway>,
        ToolTimeouts::default(),
    )
    .await;

    assert!(
        server
            .whois_lookup(Parameters(WhoisLookupParams {
                domain: "example.com".to_string(),
            }))
            .await
            .is_ok()
    );
    assert!(
        server
            .ip_lookup(Parameters(IpLookupParams {
                query: "1.1.1.1".to_string(),
            }))
            .await
            .is_ok()
    );
    assert!(
        server
            .dns_propagation_check(Parameters(DnsPropagationCheckParams {
                domain: "example.com".to_string(),
                record_type: DnsQueryType::A,
            }))
            .await
            .is_ok()
    );
    assert!(
        server
            .dnssec_check(Parameters(DnssecCheckParams {
                domain: "example.com".to_string(),
                nameserver: Some("8.8.8.8".to_string()),
            }))
            .await
            .is_ok()
    );

    assert_eq!(toolbox.whois_calls().await, vec!["example.com".to_string()]);
    assert_eq!(toolbox.ip_calls().await, vec!["1.1.1.1".to_string()]);
    assert_eq!(
        toolbox.propagation_calls().await,
        vec![("example.com".to_string(), DnsQueryType::A)]
    );
    assert_eq!(
        toolbox.dnssec_calls().await,
        vec![("example.com".to_string(), Some("8.8.8.8".to_string()))]
    );
}

#[tokio::test]
async fn get_info_contains_expected_instructions() {
    let account_repository: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());
    let server = build_server(account_repository, None, toolbox, ToolTimeouts::default()).await;

    let info = server.get_info();

    assert_eq!(info.protocol_version, ProtocolVersion::LATEST);
    let instructions = info.instructions.unwrap_or_default();
    assert!(instructions.contains("list_accounts"));
    assert!(instructions.contains("Network diagnostic tools"));
}

#[tokio::test]
async fn run_toolbox_tool_success_returns_json() {
    let future = async { Ok::<_, ToolboxError>("hello".to_string()) };
    let result = run_toolbox_tool::<String, String>(Duration::from_secs(1), future, "test tool")
        .await
        .unwrap();
    let text = &result.content[0];
    assert!(format!("{text:?}").contains("hello"));
}

#[tokio::test]
async fn run_toolbox_tool_timeout_returns_error() {
    let future = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, ToolboxError>("late".to_string())
    };
    let error = run_toolbox_tool::<String, String>(Duration::from_millis(5), future, "slow tool")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("slow tool timeout"));
}

#[tokio::test]
async fn run_toolbox_tool_error_maps_toolbox_error() {
    let future = async { Err::<String, _>(ToolboxError::NetworkError("conn refused".into())) };
    let error = run_toolbox_tool::<String, String>(Duration::from_secs(1), future, "fail tool")
        .await
        .unwrap_err();
    assert!(error.to_string().contains("conn refused"));
}
