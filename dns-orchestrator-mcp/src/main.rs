//! MCP Server entry point for DNS Orchestrator (Read-Only)
//!
//! Starts the MCP server with stdio transport, sharing credentials with the desktop app.
//!
//! # Read-Only Mode
//!
//! The MCP server operates in read-only mode - it can read accounts and domains
//! from the desktop app's storage, but cannot modify them. This ensures the
//! desktop app remains the single source of truth for data management.

mod adapters;
mod schemas;
mod server;

use std::process::ExitCode;
use std::sync::Arc;

use adapters::{KeyringCredentialStore, NoOpDomainMetadataRepository, TauriStoreAccountRepository};
use dns_orchestrator_core::services::{AccountService, DomainMetadataService, ServiceContext};
use dns_orchestrator_core::traits::{AccountRepository, InMemoryProviderRegistry};
use rmcp::ServiceExt;
use server::DnsOrchestratorMcp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize tracing to stderr (MCP uses stdout for protocol)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .without_time()
                .with_ansi(false),
        )
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting DNS Orchestrator MCP Server (read-only mode)");
    tracing::info!("MCP server shares data with desktop app but will not modify it");

    // Create adapters
    let store = KeyringCredentialStore::new();
    let credential_store = {
        tracing::info!("Credential store initialized");
        Arc::new(store)
    };

    let account_repository = Arc::new(TauriStoreAccountRepository::new());
    let provider_registry = Arc::new(InMemoryProviderRegistry::new());
    let domain_metadata_repository = Arc::new(NoOpDomainMetadataRepository::new());

    // Create service context
    let ctx = Arc::new(ServiceContext::new(
        credential_store.clone(),
        account_repository.clone(),
        provider_registry.clone(),
        domain_metadata_repository.clone(),
    ));

    // Create account service
    let account_service = Arc::new(AccountService::new(Arc::clone(&ctx)));

    // Restore accounts (bootstrap providers from stored credentials)
    tracing::info!("Restoring accounts from stored credentials...");
    let restore_result = match account_service.restore_accounts().await {
        Ok(result) => {
            tracing::info!(
                "Account restoration complete: {} succeeded, {} failed",
                result.success_count,
                result.error_count
            );
            result
        }
        Err(e) => {
            tracing::error!("Failed to restore accounts: {}", e);
            // Continue anyway - toolbox tools still work
            dns_orchestrator_core::services::RestoreResult {
                success_count: 0,
                error_count: 0,
            }
        }
    };

    // Check if all accounts failed (exit only if there were accounts and all failed)
    let accounts = match account_repository.find_all().await {
        Ok(accts) => accts,
        Err(e) => {
            tracing::error!("Failed to load accounts: {}", e);
            Vec::new()
        }
    };
    if !accounts.is_empty() && restore_result.success_count == 0 {
        tracing::error!(
            "All {} account(s) failed to restore. Check credentials and try again.",
            accounts.len()
        );
        // Don't exit - toolbox tools still work without accounts
        tracing::warn!("MCP server will continue with toolbox-only functionality");
    }

    // Create domain metadata service
    let domain_metadata_service = Arc::new(DomainMetadataService::new(domain_metadata_repository));

    // Create MCP server
    let mcp_server = DnsOrchestratorMcp::new(&ctx, account_service, domain_metadata_service);

    tracing::info!("MCP server initialized with 8 tools");

    // Start serving via stdio
    tracing::info!("Starting MCP server on stdio transport");
    let service = match mcp_server.serve(rmcp::transport::stdio()).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to start MCP server: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Wait for the server to complete
    if let Err(e) = service.waiting().await {
        tracing::error!("MCP server error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
