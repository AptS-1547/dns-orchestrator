# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

DNS Orchestrator 是一个跨平台 DNS 记录管理应用，支持桌面（macOS/Windows/Linux）和移动端（Android）。项目采用四层架构：

```
React Frontend → Tauri/Actix-web Backend → Core Library → Provider Library → DNS APIs
```

## Commands

### 开发 (Development)

```bash
# Desktop: 启动 Tauri 开发环境（热重载）
pnpm tauri dev

# Android: 启动 Android 开发
pnpm tauri android dev

# Web mode: 启动 Web 前端开发（需要 actix-web 后端运行）
pnpm dev:web
```

### 构建 (Build)

```bash
# Desktop 生产构建
pnpm tauri build

# Android 构建
pnpm tauri android build

# Web 前端构建
pnpm build:web

# 同步版本号（在发布前必须运行）
pnpm sync-version
```

### 代码质量 (Code Quality)

```bash
# 前端检查
pnpm lint              # 运行 Biome linter
pnpm format:fix        # 格式化并修复代码

# Rust 检查
pnpm lint:rust         # 运行 Clippy（所有 workspace）
pnpm format:rust       # 格式化 Rust 代码（所有 workspace）

# 完整检查
pnpm check             # 检查前端代码质量
```

### 调试 (Debug)

```bash
# 启用 Rust 调试日志
RUST_LOG=debug pnpm tauri dev

# 更详细的日志
RUST_LOG=dns_orchestrator=trace pnpm tauri dev
```

## 架构要点

### 1. Transport 抽象层

前端通过 `ITransport` 接口与后端通信，支持两种实现：

- **Tauri IPC**: Desktop/Mobile 平台使用（通过 `@tauri-apps/api` 的 `invoke`）
- **HTTP REST**: Web 平台使用（通过 `fetch`）

编译时通过 Vite alias 自动选择：
- `src/services/transport/tauri.transport.ts` - Desktop/Mobile
- `src/services/transport/http.transport.ts` - Web

**重要**: 所有后端调用必须通过 Service 层（`src/services/`），不要直接在组件中调用 transport。

### 2. Provider Library (`dns-orchestrator-provider`)

独立的 Rust crate，实现 DNS 服务商抽象：

- **Trait**: `DnsProvider` 定义统一接口（`traits.rs`）
- **Types**: 通用类型如 `DnsRecord`, `Domain`, `PaginatedResponse`（`types.rs`）
- **Error**: 统一的 `ProviderError` 错误类型（`error.rs`）
- **Factory**: `create_provider()` 和 `get_all_provider_metadata()`（`factory.rs`）

**已支持的服务商**:
- Cloudflare (`providers/cloudflare/`)
- Alibaba Cloud DNS (`providers/aliyun/`)
- DNSPod (`providers/dnspod/`)
- Huawei Cloud DNS (`providers/huaweicloud/`)

### 3. 添加新 DNS Provider

1. 在 `dns-orchestrator-provider/src/providers/your_provider/` 创建实现
2. 实现 `DnsProvider` trait 和 `ProviderErrorMapper` trait
3. 在 `providers/mod.rs` 中注册（带 feature flag）
4. 在 `factory.rs` 添加到 `create_provider()` 和 `get_all_provider_metadata()`
5. 在 `types.rs` 的 `ProviderCredentials` enum 添加凭证变体
6. 更新前端翻译文件：`src/i18n/locales/en-US.ts` 和 `zh-CN.ts`
7. （可选）在 `src/components/account/ProviderIcon.tsx` 添加图标

### 4. 凭证存储 (Credential Storage)

- **Desktop**: 系统钥匙串（macOS Keychain, Windows Credential Manager, Linux Secret Service）
- **Android**: Tauri Stronghold（因平台限制）
- **实现位置**: `src-tauri/src/adapters/credentials/`

**注意**: 凭证永远不会以明文存储在配置文件中。

### 5. 状态管理

使用 Zustand 按功能模块划分 store（`src/stores/`）：

- `accountStore.ts` - 账户管理
- `domainStore.ts` - 域名管理
- `dnsStore.ts` - DNS 记录管理
- `toolboxStore.ts` - 网络工具箱
- `settingsStore.ts` - 应用设置
- `updaterStore.ts` - 应用更新

### 6. 前端项目结构

```
src/
├── components/       # 按功能组织的 React 组件
│   ├── account/      # 账户管理 UI
│   ├── dns/          # DNS 记录管理
│   ├── domain/       # 域名管理
│   └── ui/           # 可复用 UI 组件 (shadcn/ui)
├── services/         # Service 层（调用后端）
│   ├── transport/    # Transport 抽象
│   ├── account.service.ts
│   ├── dns.service.ts
│   └── domain.service.ts
├── stores/           # Zustand 状态管理
├── types/            # TypeScript 类型定义
├── i18n/             # 国际化（en-US, zh-CN）
└── lib/              # 工具函数
```

### 7. 后端项目结构

```
src-tauri/src/
├── commands/         # Tauri command handlers (暴露给前端)
│   ├── account.rs
│   ├── dns.rs
│   ├── domain.rs
│   └── toolbox.rs
├── adapters/         # 适配器层（使用 Core Library）
│   ├── credentials/  # 凭证存储适配器
│   └── storage/      # 本地数据持久化
├── error.rs          # Tauri 错误类型
└── types.rs          # Tauri 类型定义
```

## 开发要点

### 前端 (Frontend)

- **框架**: React 19 + TypeScript 5
- **样式**: Tailwind CSS 4 + Radix UI
- **代码规范**: Biome（配置见 `biome.json`）
  - 单引号 → 双引号
  - 分号可选
  - 100 字符换行
  - 使用 `cn()` 工具函数管理 className

### 后端 (Backend)

- **Clippy 规则**: 严格模式
  - `unwrap_used`, `expect_used`, `panic` = warn
  - 禁止 `unsafe_code`
- **异步运行时**: Tokio
- **HTTP 客户端**: reqwest (feature: `rustls` for Android, `native-tls` for Desktop)
- **序列化**: serde + serde_json

### 版本同步

`pnpm sync-version` 会同步以下文件的版本号：
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`

在发布前必须运行此命令。

### 平台差异

- **Android**: 使用 `rustls` 避免 OpenSSL 交叉编译问题
- **Desktop**: 使用 `native-tls` 和系统 keyring
- **Web**: 使用 HTTP transport + actix-web 后端（WIP）

## 常见任务

### 修改后端逻辑

1. 修改 `src-tauri/src/commands/*.rs` 的 command 函数
2. 重启 `pnpm tauri dev`（Rust 不支持热重载）
3. 前端会自动调用更新后的 command

### 修改 Provider 实现

1. 编辑 `dns-orchestrator-provider/src/providers/*/`
2. 运行 `cargo test -p dns-orchestrator-provider`
3. 重启 `pnpm tauri dev`

### 添加新的 UI 组件

1. 使用 shadcn/ui: `npx shadcn add <component-name>`
2. 组件会安装到 `src/components/ui/`
3. 使用 `cn()` 工具函数管理样式

### 添加翻译

1. 编辑 `src/i18n/locales/en-US.ts` 和 `zh-CN.ts`
2. 在组件中使用 `const { t } = useTranslation()`
3. 调用 `t('key.path')`

## 注意事项

- **不要直接使用 `unwrap()`**: 在 Rust 代码中优先使用 `?` 或 `map_err()`
- **Transport 选择**: 编译时自动选择，不要手动修改 alias
- **Provider Library**: 独立 crate，可被 Tauri 和 actix-web 共享
- **凭证安全**: 永远不要记录或打印 API 密钥
- **错误处理**: 使用 `ProviderError` 和 `DnsError` 统一错误类型

