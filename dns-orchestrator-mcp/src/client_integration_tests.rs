use super::test_mocks::*;
use super::*;

use std::sync::Arc;

use dns_orchestrator_core::traits::AccountRepository;
use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Spawn a MCP server and connect a client via in-memory duplex transport.
///
/// Returns `(client, server_handle)`. The client derefs to `Peer<RoleClient>`
/// so you can call `list_all_tools()`, `call_tool()`, `peer_info()`, etc.
async fn spawn_client_server(
    account_repository: Arc<dyn AccountRepository>,
    provider: Option<Arc<MockDnsProvider>>,
    toolbox: Arc<dyn ToolboxGateway>,
    timeouts: ToolTimeouts,
) -> (
    rmcp::service::RunningService<rmcp::RoleClient, ()>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let (server_transport, client_transport) = tokio::io::duplex(4096);

    let server = build_server(account_repository, provider, toolbox, timeouts).await;

    let server_handle = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });

    let client = ().serve(client_transport).await.unwrap();
    (client, server_handle)
}

/// Spawn a default server with empty accounts and default mocks.
async fn spawn_default() -> (
    rmcp::service::RunningService<rmcp::RoleClient, ()>,
    tokio::task::JoinHandle<anyhow::Result<()>>,
) {
    let repo: Arc<dyn AccountRepository> = Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());
    spawn_client_server(repo, None, toolbox, ToolTimeouts::default()).await
}

fn call_params(name: &str, args: &serde_json::Value) -> CallToolRequestParams {
    CallToolRequestParams {
        meta: None,
        name: name.to_string().into(),
        arguments: args.as_object().cloned(),
        task: None,
    }
}

fn extract_text(result: &rmcp::model::CallToolResult) -> &str {
    result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("expected text content in result")
}

// ===========================================================================
// Scenario 1: initialize handshake
// ===========================================================================

#[tokio::test]
async fn client_connects_and_receives_server_info() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let server_info = client
        .peer_info()
        .expect("server info should be set after handshake");

    assert_eq!(server_info.protocol_version, ProtocolVersion::LATEST);
    assert!(
        server_info.capabilities.tools.is_some(),
        "server should advertise tool capability"
    );

    let instructions = server_info.instructions.as_deref().unwrap_or("");
    assert!(instructions.contains("list_accounts"));
    assert!(instructions.contains("Network diagnostic tools"));

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

// ===========================================================================
// Scenario 2: tools/list
// ===========================================================================

const EXPECTED_TOOL_NAMES: &[&str] = &[
    "list_accounts",
    "list_domains",
    "list_records",
    "dns_lookup",
    "whois_lookup",
    "ip_lookup",
    "dns_propagation_check",
    "dnssec_check",
];

#[tokio::test]
async fn tools_list_returns_all_eight_tools() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let tools = client.list_all_tools().await?;

    assert_eq!(tools.len(), 8, "expected exactly 8 tools");

    let names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    for expected in EXPECTED_TOOL_NAMES {
        assert!(names.contains(expected), "missing tool: {expected}");
    }

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn each_tool_has_description_and_object_schema() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let tools = client.list_all_tools().await?;

    for tool in &tools {
        assert!(
            tool.description.is_some(),
            "tool '{}' missing description",
            tool.name
        );

        assert_eq!(
            tool.input_schema.get("type").and_then(|v| v.as_str()),
            Some("object"),
            "tool '{}' input_schema type must be 'object'",
            tool.name
        );

        // Tools with parameters must have a `properties` key.
        // Empty-param tools (e.g. list_accounts) may omit it.
        if tool.name != "list_accounts" {
            assert!(
                tool.input_schema.contains_key("properties"),
                "tool '{}' input_schema must have 'properties'",
                tool.name
            );
        }
    }

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn tools_with_required_params_declare_them_in_schema() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let tools = client.list_all_tools().await?;
    let find_tool = |name: &str| tools.iter().find(|t| t.name == name).unwrap().clone();

    // list_domains requires account_id
    let ld = find_tool("list_domains");
    let required = ld
        .input_schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("list_domains should have required fields");
    assert!(required.iter().any(|v| v == "account_id"));

    // list_records requires account_id and domain_id
    let lr = find_tool("list_records");
    let required = lr
        .input_schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("list_records should have required fields");
    assert!(required.iter().any(|v| v == "account_id"));
    assert!(required.iter().any(|v| v == "domain_id"));

    // dns_lookup requires domain and record_type
    let dl = find_tool("dns_lookup");
    let required = dl
        .input_schema
        .get("required")
        .and_then(|v| v.as_array())
        .expect("dns_lookup should have required fields");
    assert!(required.iter().any(|v| v == "domain"));
    assert!(required.iter().any(|v| v == "record_type"));

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

// ===========================================================================
// Scenario 3: tools/call
// ===========================================================================

#[tokio::test]
async fn call_list_accounts_returns_json_array() -> anyhow::Result<()> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let toolbox = Arc::new(MockToolboxGateway::default());
    let (client, server_handle) =
        spawn_client_server(repo, None, toolbox, ToolTimeouts::default()).await;

    let result = client
        .call_tool(call_params("list_accounts", &serde_json::json!({})))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert!(parsed.is_array(), "list_accounts should return JSON array");
    assert_eq!(parsed.as_array().unwrap().len(), 1);

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_list_domains_returns_paginated_result() -> anyhow::Result<()> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let provider = Arc::new(MockDnsProvider::default());
    let toolbox = Arc::new(MockToolboxGateway::default());
    let (client, server_handle) =
        spawn_client_server(repo, Some(provider), toolbox, ToolTimeouts::default()).await;

    let result = client
        .call_tool(call_params(
            "list_domains",
            &serde_json::json!({"account_id": "acc-1"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert!(parsed.is_object());

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_list_records_returns_paginated_result() -> anyhow::Result<()> {
    let repo: Arc<dyn AccountRepository> =
        Arc::new(TestAccountRepository::new(vec![test_account("acc-1")]));
    let provider = Arc::new(MockDnsProvider::default());
    let toolbox = Arc::new(MockToolboxGateway::default());
    let (client, server_handle) =
        spawn_client_server(repo, Some(provider), toolbox, ToolTimeouts::default()).await;

    let result = client
        .call_tool(call_params(
            "list_records",
            &serde_json::json!({"account_id": "acc-1", "domain_id": "dom-1"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_dns_lookup_returns_lookup_result() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params(
            "dns_lookup",
            &serde_json::json!({"domain": "example.com", "record_type": "A"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert!(parsed.get("nameserver").is_some());
    assert!(parsed.get("records").is_some());

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_whois_lookup_returns_result() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params(
            "whois_lookup",
            &serde_json::json!({"domain": "example.com"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert_eq!(parsed["domain"], "example.com");
    assert_eq!(parsed["registrar"], "Mock Registrar");

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_ip_lookup_returns_result() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params(
            "ip_lookup",
            &serde_json::json!({"query": "1.1.1.1"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert_eq!(parsed["query"], "1.1.1.1");

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_dns_propagation_check_returns_result() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params(
            "dns_propagation_check",
            &serde_json::json!({"domain": "example.com", "record_type": "A"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert_eq!(parsed["domain"], "example.com");
    assert!(parsed.get("consistencyPercentage").is_some());

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_dnssec_check_returns_result() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params(
            "dnssec_check",
            &serde_json::json!({"domain": "example.com", "nameserver": "8.8.8.8"}),
        ))
        .await?;

    assert_ne!(result.is_error, Some(true));
    let text = extract_text(&result);
    let parsed: serde_json::Value = serde_json::from_str(text)?;
    assert_eq!(parsed["domain"], "example.com");
    assert_eq!(parsed["dnssecEnabled"], false);

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

// ===========================================================================
// Scenario 4: error handling
// ===========================================================================

#[tokio::test]
async fn call_nonexistent_tool_returns_error() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    let result = client
        .call_tool(call_params("nonexistent_tool", &serde_json::json!({})))
        .await;

    assert!(result.is_err(), "calling nonexistent tool should fail");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("tool not found"),
        "error should mention 'tool not found', got: {err}"
    );

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_tool_with_missing_required_params_returns_error() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    // list_domains requires account_id, omit it
    let result = client
        .call_tool(call_params("list_domains", &serde_json::json!({})))
        .await;

    assert!(result.is_err(), "missing required param should fail");

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_dns_lookup_with_invalid_record_type_returns_error() -> anyhow::Result<()> {
    let (client, server_handle) = spawn_default().await;

    // "INVALID" cannot deserialize to DnsQueryType
    let result = client
        .call_tool(call_params(
            "dns_lookup",
            &serde_json::json!({"domain": "example.com", "record_type": "INVALID"}),
        ))
        .await;

    assert!(
        result.is_err(),
        "invalid record_type should fail deserialization"
    );

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_tool_with_backend_failure_returns_sanitized_error() -> anyhow::Result<()> {
    let repo: Arc<dyn AccountRepository> = Arc::new(TestAccountRepository::failing_find_all());
    let toolbox = Arc::new(MockToolboxGateway::default());
    let (client, server_handle) =
        spawn_client_server(repo, None, toolbox, ToolTimeouts::default()).await;

    let result = client
        .call_tool(call_params("list_accounts", &serde_json::json!({})))
        .await;

    assert!(result.is_err(), "backend failure should return error");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("List accounts failed"),
        "error should contain sanitized message, got: {err}"
    );
    assert!(
        !err.contains("mock find_all failure"),
        "error should NOT leak internal details"
    );

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

#[tokio::test]
async fn call_toolbox_tool_with_gateway_error_returns_error() -> anyhow::Result<()> {
    let repo: Arc<dyn AccountRepository> = Arc::new(TestAccountRepository::new(Vec::new()));
    let toolbox = Arc::new(MockToolboxGateway::default());
    toolbox
        .set_dns_lookup_error(Some("connection refused".to_string()))
        .await;
    let (client, server_handle) = spawn_client_server(
        repo,
        None,
        Arc::clone(&toolbox) as Arc<dyn ToolboxGateway>,
        ToolTimeouts::default(),
    )
    .await;

    let result = client
        .call_tool(call_params(
            "dns_lookup",
            &serde_json::json!({"domain": "example.com", "record_type": "A"}),
        ))
        .await;

    assert!(result.is_err(), "toolbox error should propagate");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("connection refused"),
        "error should contain toolbox error, got: {err}"
    );

    client.cancel().await?;
    server_handle.await??;
    Ok(())
}
