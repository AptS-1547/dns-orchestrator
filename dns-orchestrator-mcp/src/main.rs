//! MCP Server entry point for DNS Orchestrator.
//!
//! Starts the MCP server with stdio transport, sharing the desktop app's
//! `SQLite` database and system keyring for credentials.

mod mcp_types;
mod schemas;
mod server;

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use dns_orchestrator_app::adapters::{KeyringCredentialStore, SqliteStore};
use dns_orchestrator_app::{AppStateBuilder, NoopStartupHooks};
use rmcp::ServiceExt;
use server::DnsOrchestratorMcp;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Current desktop application data directory identifier.
const PRIMARY_APP_DIR_NAME: &str = "net.esaps.dns-orchestrator";
/// Legacy desktop application data directory identifier kept for migration.
const LEGACY_APP_DIR_NAME: &str = "com.apts-1547.dns-orchestrator";
/// Shared SQLite filename used by both desktop and MCP server processes.
const DB_FILE_NAME: &str = "data.db";

/// Detect the Tauri desktop app's data directory.
///
/// Checks platform-specific locations for the `SQLite` database file,
/// preferring the primary app identifier over the legacy one.
fn resolve_app_data_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(data_dir) = dirs::data_dir() {
        candidates.push(data_dir.join(PRIMARY_APP_DIR_NAME));
        candidates.push(data_dir.join(LEGACY_APP_DIR_NAME));
    }

    if let Some(data_local_dir) = dirs::data_local_dir() {
        candidates.push(data_local_dir.join(PRIMARY_APP_DIR_NAME));
        candidates.push(data_local_dir.join(LEGACY_APP_DIR_NAME));
    }

    let mut seen = std::collections::HashSet::new();
    candidates.retain(|c| seen.insert(c.clone()));

    // Prefer an existing directory that already contains `data.db`.
    for candidate in &candidates {
        if candidate.join(DB_FILE_NAME).exists() {
            tracing::info!("Detected app data directory: {:?}", candidate);
            return Some(candidate.clone());
        }
    }

    // Fall back to the primary path and let `SqliteStore` create the database.
    if let Some(default) = candidates.first() {
        tracing::warn!("No existing database found, defaulting to {:?}", default);
        return Some(default.clone());
    }

    None
}

#[tokio::main]
async fn main() -> ExitCode {
    // Handle --version / -v flag
    if std::env::args().any(|a| a == "--version" || a == "-v") {
        println!("dns-orchestrator-mcp {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    // Handle --help / -h flag
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("dns-orchestrator-mcp {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("MCP (Model Context Protocol) server for DNS Orchestrator.");
        println!("Provides AI agents with DNS management capabilities across");
        println!("multiple cloud providers (Cloudflare, Aliyun, DNSPod, Huaweicloud)");
        println!("and network diagnostic tools (DNS lookup, WHOIS, DNSSEC, etc.).");
        println!();
        println!("This server communicates over stdio using the MCP protocol.");
        println!("It shares the desktop app's SQLite database and system keyring.");
        println!();
        println!("Usage: dns-orchestrator-mcp");
        println!();
        println!("Options:");
        println!("  -v, --version  Print version information");
        println!("  -h, --help     Print this help message");
        return ExitCode::SUCCESS;
    }

    // Initialize structured logs on stderr because MCP protocol frames use stdout.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .without_time()
                .with_ansi(false),
        )
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("Starting DNS Orchestrator MCP Server");

    // Resolve the database directory shared with the desktop app.
    let Some(data_dir) = resolve_app_data_dir() else {
        tracing::error!("Failed to determine app data directory");
        return ExitCode::FAILURE;
    };
    let db_path = data_dir.join(DB_FILE_NAME);

    // Create storage adapters using the same composition as the desktop app.
    let sqlite_store = match SqliteStore::new(&db_path, None).await {
        Ok(store) => Arc::new(store),
        Err(e) => {
            tracing::error!("Failed to initialize SQLite store: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let credential_store = Arc::new(KeyringCredentialStore::new());

    // Build the application state graph.
    let app_state = match AppStateBuilder::new()
        .credential_store(credential_store)
        .account_repository(sqlite_store.clone())
        .domain_metadata_repository(sqlite_store)
        .build()
    {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Failed to build app state: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Run startup routines (migrations and account restore).
    if let Err(e) = app_state.run_startup(&NoopStartupHooks).await {
        tracing::error!("Startup failed: {}", e);
        // Keep serving; toolbox-only tools can still run without account data.
    }

    // Build the MCP server from shared application services.
    let mcp_server = DnsOrchestratorMcp::new(
        &app_state.ctx,
        Arc::clone(&app_state.account_service),
        Arc::clone(&app_state.domain_metadata_service),
    );

    tracing::info!("MCP server initialized");

    // Serve MCP over stdio transport.
    let service = match mcp_server.serve(rmcp::transport::stdio()).await {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("Failed to start MCP server: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = service.waiting().await {
        tracing::error!("MCP server error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
