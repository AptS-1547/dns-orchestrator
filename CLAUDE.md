# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DNS Orchestrator 是一个跨平台的 DNS 记录管理工具，支持多个 DNS 服务商的统一管理。项目采用 Tauri 2 + React 19 架构，使用 Rust 编写后端，TypeScript 编写前端。

**核心特性**：

- 多账户管理（Cloudflare、阿里云、腾讯云 DNSPod、华为云）
- DNS 记录 CRUD 操作（分页、搜索、过滤）
- 网络工具箱（DNS 查询、WHOIS、IP 查询、SSL 检查）
- 账户导入导出（AES-GCM 加密）
- 跨平台支持（macOS、Windows、Linux、Android）

## 项目架构

### 四层架构模式

```text
Frontend (React + Zustand)
    ↓ Transport Abstraction (Tauri IPC / HTTP)
Backend (Tauri Commands / Actix-web)
    ↓ ServiceContext (DI Container)
Core Library (dns-orchestrator-core)
    ↓ DnsProvider Trait
Provider Library (dns-orchestrator-provider)
    ↓ HTTPS
DNS Provider APIs
```

### Workspace 结构

项目使用 Rust workspace + pnpm workspace 管理多个包：

- **`src/`**: React 前端（TypeScript）
- **`dns-orchestrator-core/`**: 平台无关的核心业务逻辑（Rust crate）
- **`dns-orchestrator-provider/`**: DNS 服务商抽象库（Rust crate，可独立复用）
- **`src-tauri/`**: Tauri 桌面/移动端后端（Rust）
- **`src-actix-web/`**: Web 后端（Actix-web，开发中）

### 依赖注入模式

Core 库使用 trait 抽象平台特定实现：

- **`CredentialStore`**: 凭据存储（Desktop 用 keyring，Android 用 Stronghold）
- **`AccountRepository`**: 账户元数据存储（tauri-plugin-store 或 SeaORM）
- **`ProviderRegistry`**: 运行时 provider 实例管理（内存 HashMap）

Tauri 后端在 `src-tauri/src/adapters/` 中实现这些 trait。

### Transport 抽象

前端通过 `ITransport` 接口调用后端，编译时根据 `VITE_PLATFORM` 选择：

- **Desktop/Mobile**: `TauriTransport` → Tauri IPC
- **Web**: `HttpTransport` → REST API

## 常用命令

### 开发模式

```bash
# 桌面端开发（带热重载）
pnpm tauri dev

# Android 开发
pnpm tauri android dev

# Web 模式（需要先启动 actix-web 后端）
pnpm dev:web
```

### 构建

```bash
# 桌面端生产构建
pnpm tauri build

# Android 构建
pnpm tauri android build

# Web 前端构建
pnpm build:web
```

### 代码质量

```bash
# 前端代码检查和格式化
pnpm lint              # Biome lint
pnpm format:fix        # Biome format + lint fix
pnpm check             # 运行所有检查

# Rust 代码检查和格式化
pnpm format:rust       # cargo fmt (所有 workspace)
pnpm lint:rust         # cargo clippy (所有 workspace)
pnpm lint:rust:fix     # clippy --fix

# 单独运行特定 workspace
pnpm format:rust:tauri
pnpm lint:rust:provider
```

### 版本同步

```bash
# 同步版本号到所有配置文件
pnpm sync-version
```

这会更新：
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src-actix-web/Cargo.toml`

**重要**: 发布前必须运行此命令！

### 测试

```bash
# Rust 测试
cargo test -p dns-orchestrator-provider  # Provider 库测试
cargo test -p dns-orchestrator-core      # Core 库测试
cargo test --workspace                   # 所有 Rust 测试
```

## 常见开发任务

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands/*.rs` 中添加命令函数（使用 `#[tauri::command]` 宏）
2. 在 `src-tauri/src/commands/mod.rs` 中导出
3. 在 `src-tauri/src/lib.rs` 的 `invoke_handler![]` 中注册
4. 在 `src/services/transport/types.ts` 的 `CommandMap` 接口中添加类型定义
5. 在对应的前端 service (`src/services/*.service.ts`) 中添加调用方法
6. 在 store 或组件中使用

### 修改 DNS 记录的数据结构

1. **Backend**: 修改 `dns-orchestrator-provider/src/types.rs` 的 `DnsRecord` 或 `RecordData` 枚举
2. **Provider**: 更新各 provider 实现（`cloudflare/`, `aliyun/`, `dnspod/`, `huaweicloud/`）
3. **Frontend**: 同步修改 `src/types/*.ts` 中的对应类型
4. **UI**: 更新 `src/components/dns/` 中的相关组件

### 添加新的网络工具

1. **Core**: 在 `dns-orchestrator-core/src/services/toolbox/` 添加 service
2. **Tauri**: 在 `src-tauri/src/commands/toolbox.rs` 添加命令
3. **Frontend**: 在 `src/components/toolbox/` 添加 UI 组件
4. **Store**: 在 `src/stores/toolboxStore.ts` 添加状态管理
5. **i18n**: 在 `src/i18n/locales/` 添加翻译

### 修改分页逻辑

**后端**:

- `dns-orchestrator-provider/src/types.rs`: `PaginationParams` / `RecordQueryParams`
- 各 provider 的 `list_records()` 实现

**前端**:

- `src/stores/dnsStore.ts`: `fetchRecords()`, `currentPage`, `pageSize`
- `src/components/dns/DnsRecordTable.tsx`: 分页 UI

### 更改凭据存储方式

1. 修改 `dns-orchestrator-core/src/traits/credential_store.rs` trait 定义
2. 更新 `src-tauri/src/adapters/credential_store.rs` 实现
3. 如需支持 Web，更新 `src-actix-web/` 的 SeaORM 实现

### 添加新的 UI 组件（shadcn/ui）

```bash
# 安装 shadcn/ui 组件
npx shadcn@latest add <component-name>
```

组件会添加到 `src/components/ui/`，使用 Radix UI + Tailwind CSS。

### 调试 Provider API 调用

1. 启用详细日志: `RUST_LOG=dns_orchestrator_provider=trace pnpm tauri dev`
2. 在 `dns-orchestrator-provider/src/providers/<provider>/http.rs` 添加日志
3. 使用 `reqwest` 的 debug middleware 查看请求/响应

## 开发指南

### 添加新的 DNS Provider

所有 provider 实现在 `dns-orchestrator-provider/` 中（v1.1.0+ 的设计）。

详细步骤参考 `docs/DEVELOPMENT.md` 的 "Adding a New DNS Provider" 章节。

关键步骤：

1. 在 `dns-orchestrator-provider/src/providers/` 创建 provider 模块
2. 实现 `DnsProvider` trait
3. 在 `factory.rs` 注册
4. 添加 feature flag 和前端翻译

### 前端状态管理

使用 Zustand，每个功能域一个 store：

- **accountStore**: 账户列表、providers、当前账户
- **domainStore**: 域名列表（按账户分组）
- **dnsStore**: DNS 记录、分页、搜索、过滤、批量选择
- **toolboxStore**: 工具箱历史记录
- **settingsStore**: 主题、语言、调试模式

**最佳实践**: 使用 `useShallow` 优化重渲染：

```typescript
const { records, hasMore } = useDnsStore(useShallow(state => ({
  records: state.records,
  hasMore: state.hasMore,
})))
```

### 错误处理

- **Provider 错误**: 使用 `ProviderError` 枚举（13 种变体），映射所有 provider 错误
- **Core 错误**: 使用 `CoreError` 枚举
- **前端**: 通过 `ApiResponse<T>` 统一返回格式

### Git Commit 规范

使用 Conventional Commits：

```
feat: 新功能
fix: Bug 修复
docs: 文档更新
style: 代码格式
refactor: 重构
perf: 性能优化
test: 测试
chore: 构建/工具链
```

Scope 示例：`toolbox`, `provider`, `core`, `ui`, `android`

## 故障排查

### "Failed to load credentials"

- **Desktop**: 确保系统 keychain 服务运行
  - macOS: Keychain Access 正常
  - Linux: `gnome-keyring` 或 `kwallet` 运行中
  - Windows: Credential Manager 可访问
- **Android**: 检查 Stronghold 初始化日志

### Provider API 错误

1. 启用 debug 日志: `RUST_LOG=dns_orchestrator_provider=debug pnpm tauri dev`
2. 检查凭据是否正确（在 provider 控制台验证）
3. 查看 `ProviderError` 的具体变体和 `raw_message`
4. 确认 API quota 未超限

### 构建失败

```bash
# 清理缓存重新构建
cargo clean
rm -rf node_modules pnpm-lock.yaml
pnpm install
pnpm tauri dev
```

### Android 构建问题

- **OpenSSL 错误**: 确保使用 `rustls` feature（`dns-orchestrator-provider` 和 `dns-orchestrator-core` 的 Android target dependencies）
- **NDK 版本**: 确认使用 NDK r26b
- **签名错误**: 检查 `SIGNING_*` 环境变量和 keystore 路径

### macOS 签名/公证失败

- **证书**: 检查 `security find-identity -v -p codesigning`
- **公证**: 确认 App Store Connect API key 配置正确
- **权限**: 查看 `src-tauri/capabilities/` 是否声明了所有需要的权限

## 平台特定注意事项

### Android

- **TLS**: 使用 `rustls` 而非 `native-tls`（避免 OpenSSL 交叉编译问题）
- **凭据存储**: 使用 `tauri-plugin-stronghold` 而非 `keyring`
- **更新器**: 自定义实现（`src-tauri/src/commands/updater.rs`）

### macOS

- **签名**: 使用 Developer ID Application 证书
- **公证**: 通过 App Store Connect API 自动完成
- **窗口**: 使用自定义 titlebar（`data-tauri-drag-region`）

### Linux

- **依赖**: webkit2gtk-4.1, libappindicator3, Secret Service (gnome-keyring/kwallet)
- **构建**: 支持 x64 和 ARM64（通过 GitHub Actions 原生 runner）

## 代码风格

### TypeScript/React

- **Formatter**: Biome（100 字符行宽，双引号，分号可选）
- **组件组织**: 按功能域（account, dns, domain, toolbox）
- **导入顺序**: React → 第三方库 → @/ 别名 → 相对路径

### Rust

- **Formatter**: rustfmt（默认配置）
- **Linter**: Clippy（pedantic 级别，见各 Cargo.toml）
- **严格规则**:
  - `unsafe_code = "forbid"`
  - `unwrap_used = "warn"`（必须处理错误）
  - `expect_used = "warn"`

## CI/CD

### GitHub Actions Workflow

- **触发方式**:
  - Tag push (`v*`): 自动构建所有平台并创建 draft release
  - Manual dispatch: 选择特定平台构建

- **支持平台**:
  - macOS (Apple Silicon + Intel)
  - Windows (x64 + ARM64)
  - Linux (x64 + ARM64)
  - Android (universal APK)
  - iOS (TestFlight)

- **自动化**:
  - macOS 签名和公证
  - Android APK 签名
  - 生成 `latest.json` 用于 Tauri 自动更新
  - 创建 release notes（从 `docs/release-notes/v*.md`）

### 发布流程

1. 更新版本号：`pnpm sync-version`
2. 创建 release notes: `docs/release-notes/v1.x.x.md`
3. 提交并打 tag: `git tag v1.x.x && git push origin v1.x.x`
4. GitHub Actions 自动构建所有平台
5. 检查 draft release，编辑并发布

**如果 CI 失败**:

- 查看失败的 job 日志
- 可以删除 tag 重新推送: `git tag -d v1.x.x && git push origin :refs/tags/v1.x.x`
- 或使用 Manual dispatch 单独构建失败的平台

**回滚发布**:

- 标记 release 为 pre-release
- 推送新 tag 发布修复版本
- Tauri updater 会自动跳过 pre-release

## 重要文件

- **`vite.config.ts`**: 平台感知构建配置（transport 别名）
- **`src-tauri/tauri.conf.json`**: Tauri 配置（权限、bundle、updater）
- **`src-tauri/capabilities/`**: Tauri 2 权限清单
- **`biome.json`**: Biome 配置
- **`src/services/transport/types.ts`**: 类型安全的 CommandMap（所有 24 个命令）

## 调试技巧

### 前端调试

```bash
# 打开 DevTools
macOS: Cmd+Option+I
Windows/Linux: F12 或 Ctrl+Shift+I
```

### 后端调试

```bash
# 启用 Rust 日志
RUST_LOG=debug pnpm tauri dev
RUST_LOG=dns_orchestrator=trace pnpm tauri dev  # 更详细

# 查看特定模块
RUST_LOG=dns_orchestrator::commands::dns=trace pnpm tauri dev
```

**使用 rust-lldb 调试** (macOS/Linux):

```bash
# 构建 debug 版本
cargo build --manifest-path src-tauri/Cargo.toml

# 启动调试器
rust-lldb target/debug/dns-orchestrator
# 设置断点: b dns_orchestrator::commands::dns::list_dns_records
# 运行: r
```

**使用 VS Code 调试**:

安装 CodeLLDB 插件，在 `.vscode/launch.json` 配置 Rust 调试。

## 性能优化

- **分页**: 服务端分页（默认 20 条/页）
- **搜索防抖**: 300ms debounce
- **无限滚动**: IntersectionObserver
- **内存缓存**: 凭据和账户缓存（RwLock）
- **Async/Await**: Tokio 异步运行时
- **Feature Flags**: 按需编译 providers

## 安全性

- **凭据存储**: 系统 keychain（macOS Keychain、Windows Credential Manager、Linux Secret Service）
- **导入导出**: AES-GCM 加密 + PBKDF2 密钥派生
- **HTTPS**: 所有 API 调用使用 TLS
- **Tauri 权限**: 最小权限原则（capabilities/）

## 参考文档

- **架构详解**: `docs/ARCHITECTURE.md`
- **开发指南**: `docs/DEVELOPMENT.md`
- **Tauri 文档**: <https://v2.tauri.app/>
- **React 文档**: <https://react.dev/>
