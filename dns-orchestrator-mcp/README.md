# dns-orchestrator-mcp

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for DNS Orchestrator, exposing read-only DNS management tools to AI agents.

Shares account data with the desktop app but operates in read-only mode -- the desktop app remains the single source of truth.

## Features

- **Account Management Tools** -- List accounts, domains, and DNS records with pagination
- **Network Diagnostic Tools** -- DNS lookup, WHOIS, IP geolocation, DNS propagation check, DNSSEC validation
- **Shared Storage** -- Reads accounts from Tauri Store and credentials from system keyring
- **Security** -- Sanitized error messages, timeout protection, resource limits
- **Stateless** -- No write operations, no data modification

## Available Tools

| Tool | Description |
|------|-------------|
| `list_accounts` | List all configured DNS provider accounts (Cloudflare, Aliyun, DNSPod, Huaweicloud) |
| `list_domains` | List domains for a specific account with pagination |
| `list_records` | List DNS records for a domain with filtering and pagination |
| `dns_lookup` | Perform DNS lookup (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL) |
| `whois_lookup` | Query WHOIS information (registrar, dates, name servers) |
| `ip_lookup` | Look up IP geolocation (country, region, city, ISP, ASN) |
| `dns_propagation_check` | Check DNS record propagation across 13 global servers |
| `dnssec_check` | Validate DNSSEC deployment (DNSKEY, DS, RRSIG records) |

## Usage

### Build

```bash
cargo build --release
```

The executable will be at `target/release/dns-orchestrator-mcp`.

### Configure MCP Client

Add to your MCP client configuration (e.g., Claude Desktop):

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "dns-orchestrator": {
      "command": "/path/to/dns-orchestrator-mcp"
    }
  }
}
```

### Run

The server will:
1. Load account configurations from Tauri Store (`~/.dns-orchestrator/accounts.json`)
2. Restore credentials from system keyring
3. Initialize DNS provider connections
4. Start MCP server on stdio transport

Network toolbox tools work without any account configuration.

## Architecture

```
┌─────────────────────────────────────┐
│      MCP Client (Claude, etc.)      │
└──────────────┬──────────────────────┘
               │ stdio transport
┌──────────────▼──────────────────────┐
│     dns-orchestrator-mcp server     │
│  8 read-only tools (list/lookup)    │
└──────────────┬──────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────────────┐  ┌────▼──────────────┐
│ Tauri Store    │  │ System Keyring    │
│ (accounts.json)│  │ (credentials)     │
└────────────────┘  └───────────────────┘
```

### Data Sharing

- **Account Repository**: `TauriStoreAccountRepository` reads from `~/.dns-orchestrator/accounts.json`
- **Credential Store**: `KeyringCredentialStore` reads from system keyring (same service as desktop app)
- **Domain Metadata**: `NoOpDomainMetadataRepository` (MCP doesn't need persistent metadata)

### Security

- Error messages sanitized to prevent credential leakage
- Full errors logged to stderr, generic messages returned to client
- Timeout limits on all external service calls (15-60 seconds)
- Page size clamped to max 100 items

## Development

### Run Tests

```bash
cargo test
```

### Logging

Set log level via `RUST_LOG` environment variable:

```bash
RUST_LOG=debug dns-orchestrator-mcp
```

Logs are written to stderr (MCP protocol uses stdout).

## License

MIT
