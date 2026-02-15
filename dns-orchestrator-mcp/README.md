# dns-orchestrator-mcp

[Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server for [DNS Orchestrator](https://github.com/AptS-1547/dns-orchestrator), exposing DNS management and network diagnostic tools to AI agents.

Shares account data with the DNS Orchestrator desktop app via SQLite database and system keyring, operating in **read-only mode**.

## Install

### From crates.io

```bash
cargo install dns-orchestrator-mcp
```

### From source

```bash
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator
cargo build --release -p dns-orchestrator-mcp
# Binary: target/release/dns-orchestrator-mcp
```

## Configure MCP Client

Add to your MCP client configuration:

### Claude Desktop

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "dns-orchestrator": {
      "command": "dns-orchestrator-mcp"
    }
  }
}
```

### Claude Code

```bash
claude mcp add dns-orchestrator dns-orchestrator-mcp
```

### Cursor / Windsurf

Add to `.cursor/mcp.json` or MCP settings:

```json
{
  "mcpServers": {
    "dns-orchestrator": {
      "command": "dns-orchestrator-mcp"
    }
  }
}
```

> If `dns-orchestrator-mcp` is not in your `PATH`, use the full path to the binary (e.g. `~/.cargo/bin/dns-orchestrator-mcp`).

## Available Tools

### Account & DNS Management

Requires the [DNS Orchestrator desktop app](https://github.com/AptS-1547/dns-orchestrator) to be installed with accounts configured.

| Tool | Description |
|------|-------------|
| `list_accounts` | List all configured DNS provider accounts (Cloudflare, Aliyun, DNSPod, Huaweicloud) |
| `list_domains` | List domains for a specific account with pagination |
| `list_records` | List DNS records for a domain with filtering and pagination |

### Network Diagnostics

Work standalone without any account configuration.

| Tool | Description |
|------|-------------|
| `dns_lookup` | DNS lookup (A, AAAA, CNAME, MX, TXT, NS, SOA, SRV, CAA, PTR, ALL) |
| `whois_lookup` | WHOIS query (registrar, dates, name servers) |
| `ip_lookup` | IP geolocation (country, region, city, ISP, ASN) |
| `dns_propagation_check` | Check DNS propagation across 13 global servers |
| `dnssec_check` | Validate DNSSEC deployment (DNSKEY, DS, RRSIG records) |

## Data Sharing with Desktop App

The MCP server shares data with the DNS Orchestrator desktop app:

- **SQLite database** (`data.db`) — account configurations and domain metadata
- **System keyring** — provider credentials (API keys/tokens)

The server auto-detects the database from the desktop app's data directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/net.esaps.dns-orchestrator/data.db` |
| Linux | `~/.local/share/net.esaps.dns-orchestrator/data.db` |
| Windows | `%APPDATA%\net.esaps.dns-orchestrator\data.db` |

If the database doesn't exist, the server creates it automatically. Network diagnostic tools work without any database.

## Logging

Logs are written to stderr (MCP protocol uses stdout).

```bash
RUST_LOG=debug dns-orchestrator-mcp
```

## License

MIT
