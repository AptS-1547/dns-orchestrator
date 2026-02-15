# dns-orchestrator-mcp

为 [DNS Orchestrator](https://github.com/AptS-1547/dns-orchestrator) 提供的 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 服务器，向 AI 代理暴露 DNS 管理和网络诊断工具。

与 DNS Orchestrator 桌面应用通过 SQLite 数据库和系统密钥环共享账户数据，以**只读模式**运行。

## 安装

### 从 crates.io

```bash
cargo install dns-orchestrator-mcp
```

### 从源码构建

```bash
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator
cargo build --release -p dns-orchestrator-mcp
# 可执行文件: target/release/dns-orchestrator-mcp
```

## 配置 MCP 客户端

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

添加到 `.cursor/mcp.json` 或 MCP 设置：

```json
{
  "mcpServers": {
    "dns-orchestrator": {
      "command": "dns-orchestrator-mcp"
    }
  }
}
```

> 如果 `dns-orchestrator-mcp` 不在 `PATH` 中，请使用完整路径（如 `~/.cargo/bin/dns-orchestrator-mcp`）。

## 可用工具

### 账户与 DNS 管理

需要先安装 [DNS Orchestrator 桌面应用](https://github.com/AptS-1547/dns-orchestrator) 并配置好账户。

| 工具 | 描述 |
|------|------|
| `list_accounts` | 列出所有配置的 DNS 提供商账户（Cloudflare、阿里云、DNSPod、华为云） |
| `list_domains` | 列出指定账户的域名，支持分页 |
| `list_records` | 列出域名的 DNS 记录，支持过滤和分页 |

### 网络诊断

无需任何账户配置即可独立使用。

| 工具 | 描述 |
|------|------|
| `dns_lookup` | DNS 查询（A、AAAA、CNAME、MX、TXT、NS、SOA、SRV、CAA、PTR、ALL） |
| `whois_lookup` | WHOIS 查询（注册商、日期、名称服务器） |
| `ip_lookup` | IP 地理位置查询（国家、地区、城市、ISP、ASN） |
| `dns_propagation_check` | 检查 DNS 记录在全球 13 个服务器上的传播情况 |
| `dnssec_check` | 验证 DNSSEC 部署（DNSKEY、DS、RRSIG 记录） |

## 与桌面应用的数据共享

MCP 服务器与 DNS Orchestrator 桌面应用共享以下数据：

- **SQLite 数据库**（`data.db`）— 账户配置和域名元数据
- **系统密钥环** — 提供商凭证（API 密钥/令牌）

服务器会自动检测桌面应用的数据目录：

| 平台 | 路径 |
|------|------|
| macOS | `~/Library/Application Support/net.esaps.dns-orchestrator/data.db` |
| Linux | `~/.local/share/net.esaps.dns-orchestrator/data.db` |
| Windows | `%APPDATA%\net.esaps.dns-orchestrator\data.db` |

如果数据库不存在，服务器会自动创建。网络诊断工具无需任何数据库即可使用。

## 日志

日志写入 stderr（MCP 协议使用 stdout）。

```bash
RUST_LOG=debug dns-orchestrator-mcp
```

## 许可证

MIT
