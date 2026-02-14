# dns-orchestrator-mcp

为 DNS Orchestrator 提供 [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) 服务器，向 AI 代理暴露只读的 DNS 管理工具。

与桌面应用共享账户数据，但以只读模式运行 -- 桌面应用始终是唯一的数据源。

## 功能特性

- **账户管理工具** -- 列出账户、域名和 DNS 记录，支持分页
- **网络诊断工具** -- DNS 查询、WHOIS、IP 地理位置、DNS 传播检查、DNSSEC 验证
- **共享存储** -- 从 Tauri Store 读取账户，从系统密钥环读取凭证
- **安全性** -- 清理错误消息、超时保护、资源限制
- **无状态** -- 无写操作，不修改数据

## 可用工具

| 工具 | 描述 |
|------|------|
| `list_accounts` | 列出所有配置的 DNS 提供商账户（Cloudflare、阿里云、DNSPod、华为云） |
| `list_domains` | 列出指定账户的域名，支持分页 |
| `list_records` | 列出域名的 DNS 记录，支持过滤和分页 |
| `dns_lookup` | 执行 DNS 查询（A、AAAA、CNAME、MX、TXT、NS、SOA、SRV、CAA、PTR、ALL） |
| `whois_lookup` | 查询 WHOIS 信息（注册商、日期、名称服务器） |
| `ip_lookup` | 查询 IP 地理位置（国家、地区、城市、ISP、ASN） |
| `dns_propagation_check` | 检查 DNS 记录在全球 13 个服务器上的传播情况 |
| `dnssec_check` | 验证 DNSSEC 部署（DNSKEY、DS、RRSIG 记录） |

## 使用方法

### 构建

```bash
cargo build --release
```

可执行文件位于 `target/release/dns-orchestrator-mcp`。

### 配置 MCP 客户端

在 MCP 客户端配置中添加（例如 Claude Desktop）：

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

### 运行

服务器将：
1. 从 Tauri Store 加载账户配置（`~/.dns-orchestrator/accounts.json`）
2. 从系统密钥环恢复凭证
3. 初始化 DNS 提供商连接
4. 在 stdio 传输上启动 MCP 服务器

即使没有配置账户，网络工具箱功能仍然可用。

## 架构

```
┌─────────────────────────────────────┐
│      MCP 客户端（Claude 等）         │
└──────────────┬──────────────────────┘
               │ stdio 传输
┌──────────────▼──────────────────────┐
│     dns-orchestrator-mcp 服务器     │
│  8 个只读工具（list/lookup）         │
└──────────────┬──────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼────────────┐  ┌────▼──────────────┐
│ Tauri Store    │  │ 系统密钥环         │
│ (accounts.json)│  │ (凭证)            │
└────────────────┘  └───────────────────┘
```

### 数据共享

- **账户仓库**: `TauriStoreAccountRepository` 从 `~/.dns-orchestrator/accounts.json` 读取
- **凭证存储**: `KeyringCredentialStore` 从系统密钥环读取（与桌面应用使用相同服务）
- **域名元数据**: `NoOpDomainMetadataRepository`（MCP 不需要持久化元数据）

### 安全性

- 错误消息经过清理，防止凭证泄露
- 完整错误记录到 stderr，向客户端返回通用消息
- 所有外部服务调用设置超时限制（15-60 秒）
- 分页大小限制为最大 100 项

## 开发

### 运行测试

```bash
cargo test
```

### 日志

通过 `RUST_LOG` 环境变量设置日志级别：

```bash
RUST_LOG=debug dns-orchestrator-mcp
```

日志写入 stderr（MCP 协议使用 stdout）。

## 许可证

MIT
