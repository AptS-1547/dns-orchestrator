# 开发指南

本指南将帮助您设置开发环境并理解代码库结构，以便为 DNS Orchestrator 做出贡献。

## 目录

- [前置要求](#前置要求)
- [快速开始](#快速开始)
- [项目结构](#项目结构)
- [开发工作流](#开发工作流)
- [添加新的 DNS 服务商](#添加新的-dns-服务商)
- [构建与发布](#构建与发布)
- [测试](#测试)
- [常见问题](#常见问题)

## 前置要求

### 必需工具

- **Node.js**: 22+（推荐使用 LTS 版本）
- **pnpm**: 10+（包管理器）
- **Rust**: 最新稳定版（通过 [rustup](https://rustup.rs/) 安装）
- **Git**: 用于版本控制

### 平台特定依赖

#### macOS
```bash
xcode-select --install
```

#### Windows
安装 [Visual Studio Build Tools](https://visualstudio.microsoft.com/zh-hans/downloads/)，选择 C++ 开发工具。

#### Linux (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  xdg-utils \
  build-essential \
  curl \
  wget
```

其他发行版请参阅 [Tauri 前置要求](https://tauri.app/v2/guides/prerequisites/)。

## 快速开始

### 克隆仓库

```bash
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator
```

### 安装依赖

```bash
# 安装前端依赖
pnpm install

# Rust 依赖由 Cargo 管理，首次构建时会自动安装
```

### 启动开发服务器

```bash
# 以开发模式启动 Tauri，支持热重载
pnpm tauri dev
```

这将会：
1. 启动 Vite 开发服务器（React 前端）
2. 编译 Rust 后端
3. 启动应用窗口并启用热重载

### 生产构建

```bash
# 构建优化的生产版本
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 项目结构

```
dns-orchestrator/
├── src/                          # 前端 (React + TypeScript)
│   ├── components/               # React 组件
│   │   ├── account/              # 账号管理 UI
│   │   ├── dns/                  # DNS 记录管理
│   │   ├── domain/               # 域名管理
│   │   ├── toolbox/              # 网络工具箱 (DNS/WHOIS)
│   │   ├── settings/             # 设置页面
│   │   └── ui/                   # 可复用 UI 组件
│   ├── stores/                   # Zustand 状态管理
│   │   ├── accountStore.ts       # 账号状态
│   │   ├── dnsStore.ts           # DNS 记录状态
│   │   ├── domainStore.ts        # 域名状态
│   │   ├── toolboxStore.ts       # 工具箱状态
│   │   └── settingsStore.ts      # 应用设置
│   ├── types/                    # TypeScript 类型定义
│   │   ├── account.ts
│   │   ├── dns.ts
│   │   ├── domain.ts
│   │   ├── provider.ts
│   │   └── toolbox.ts
│   ├── i18n/                     # 国际化
│   │   ├── index.ts
│   │   └── locales/
│   │       ├── en-US.ts          # 英文翻译
│   │       └── zh-CN.ts          # 中文翻译
│   ├── App.tsx                   # 根组件
│   ├── main.tsx                  # React 入口
│   └── index.css                 # 全局样式
│
├── src-tauri/                    # 后端 (Rust + Tauri)
│   ├── src/
│   │   ├── commands/             # Tauri 命令处理器
│   │   │   ├── account.rs        # 账号管理命令
│   │   │   ├── dns.rs            # DNS 操作
│   │   │   ├── domain.rs         # 域名操作
│   │   │   └── toolbox.rs        # 网络工具箱命令
│   │   ├── providers/            # DNS 服务商实现
│   │   │   ├── mod.rs            # Provider trait 和注册表
│   │   │   ├── cloudflare.rs
│   │   │   ├── aliyun.rs
│   │   │   ├── dnspod.rs
│   │   │   └── huaweicloud.rs
│   │   ├── credentials/          # 安全凭证存储
│   │   │   ├── mod.rs
│   │   │   └── keychain.rs       # 系统钥匙串集成
│   │   ├── storage/              # 本地数据持久化
│   │   │   ├── mod.rs
│   │   │   └── account_store.rs
│   │   ├── crypto.rs             # 加密工具
│   │   ├── error.rs              # 错误类型和处理
│   │   ├── types.rs              # Rust 类型定义
│   │   ├── lib.rs                # Tauri 库入口
│   │   └── main.rs               # 应用入口
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 配置
│   └── build.rs                  # 构建脚本
│
├── .github/
│   └── workflows/
│       └── release.yml           # GitHub Actions 发布工作流
├── package.json                  # 前端依赖和脚本
├── vite.config.ts                # Vite 配置
├── tsconfig.json                 # TypeScript 配置
└── README.md
```

### 关键组件

#### 前端
- **Components**: 按功能组织（account, dns, domain, toolbox）
- **Stores**: Zustand stores 用于状态管理（每个功能域一个）
- **Types**: 与 Rust 后端类型匹配的共享 TypeScript 接口
- **i18n**: 英文和中文翻译文件

#### 后端
- **Commands**: Tauri 命令处理器，通过 `invoke()` 暴露给前端
- **Providers**: 遵循 `DnsProvider` trait 的 DNS 服务商实现
- **Credentials**: 系统钥匙串集成，用于安全存储
- **Storage**: 基于 JSON 的本地账号元数据存储

## 开发工作流

### 热重载

开发服务器支持热模块替换 (HMR)：
- **前端更改**：即时重载，不丢失状态
- **后端更改**：需要手动重启 `pnpm tauri dev`

### 调试

#### 前端调试
在应用窗口中打开开发者工具：
- **macOS/Linux**: `Cmd+Option+I` 或 `Ctrl+Shift+I`
- **Windows**: `F12`

#### 后端调试
使用 `log` crate 添加日志：

```rust
use log::{info, warn, error};

info!("这是一条信息");
warn!("这是一个警告");
error!("这是一个错误");
```

启用日志运行：
```bash
RUST_LOG=debug pnpm tauri dev
```

### 版本同步

项目使用自定义脚本保持版本同步：

```bash
pnpm sync-version
```

这将更新：
- `package.json` → `version`
- `src-tauri/tauri.conf.json` → `version`
- `src-tauri/Cargo.toml` → `version`

创建发布前务必运行此命令。

## 添加新的 DNS 服务商

本节将指导您添加对新 DNS 服务商的支持。

### 步骤 1：创建服务商实现

在 `src-tauri/src/providers/your_provider.rs` 创建新文件：

```rust
use async_trait::async_trait;
use reqwest::Client;
use std::collections::HashMap;

use crate::error::{DnsError, Result};
use crate::providers::DnsProvider;
use crate::types::*;

pub struct YourProvider {
    client: Client,
    credentials: HashMap<String, String>,
}

impl YourProvider {
    pub fn new(credentials: HashMap<String, String>) -> Self {
        Self {
            client: Client::new(),
            credentials,
        }
    }

    fn get_credential(&self, key: &str) -> Result<String> {
        self.credentials
            .get(key)
            .cloned()
            .ok_or_else(|| DnsError::MissingCredential(key.to_string()))
    }
}

#[async_trait]
impl DnsProvider for YourProvider {
    fn id(&self) -> &'static str {
        "your_provider"
    }

    async fn validate_credentials(&self) -> Result<bool> {
        // 实现凭证验证
        // 进行一个简单的 API 调用来验证凭证是否有效
        todo!()
    }

    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>> {
        // 实现域名列表获取（带分页）
        todo!()
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain> {
        // 实现获取单个域名详情
        todo!()
    }

    async fn list_records(
        &self,
        domain_id: &str,
        params: &RecordQueryParams,
    ) -> Result<PaginatedResponse<DnsRecord>> {
        // 实现 DNS 记录列表获取（带分页和过滤）
        todo!()
    }

    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord> {
        // 实现 DNS 记录创建
        todo!()
    }

    async fn update_record(
        &self,
        record_id: &str,
        req: &UpdateDnsRecordRequest,
    ) -> Result<DnsRecord> {
        // 实现 DNS 记录更新
        todo!()
    }

    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()> {
        // 实现 DNS 记录删除
        todo!()
    }
}
```

### 步骤 2：注册服务商

更新 `src-tauri/src/providers/mod.rs`：

```rust
mod your_provider;
pub use your_provider::YourProvider;

// 在 create_provider 函数中：
pub fn create_provider(
    provider_type: &str,
    credentials: HashMap<String, String>,
) -> Result<Arc<dyn DnsProvider>> {
    match provider_type {
        "cloudflare" => Ok(Arc::new(CloudflareProvider::new(credentials))),
        "aliyun" => Ok(Arc::new(AliyunProvider::new(credentials))),
        "dnspod" => Ok(Arc::new(DnspodProvider::new(credentials))),
        "huaweicloud" => Ok(Arc::new(HuaweicloudProvider::new(credentials))),
        "your_provider" => Ok(Arc::new(YourProvider::new(credentials))), // 添加这一行
        _ => Err(DnsError::ProviderNotFound(provider_type.to_string())),
    }
}

// 在 get_all_provider_metadata() 中添加服务商元数据：
ProviderMetadata {
    id: "your_provider".to_string(),
    name: "你的服务商".to_string(),
    description: "你的 DNS 服务商描述".to_string(),
    required_fields: vec![
        ProviderCredentialField {
            key: "apiKey".to_string(),
            label: "API Key".to_string(),
            field_type: "password".to_string(),
            placeholder: Some("输入 API Key".to_string()),
            help_text: Some("从服务商控制台获取".to_string()),
        }
    ],
    features: ProviderFeatures::default(),
},
```

### 步骤 3：添加前端类型

更新 `src/types/provider.ts`：

```typescript
export type ProviderType =
  | 'cloudflare'
  | 'aliyun'
  | 'dnspod'
  | 'huaweicloud'
  | 'your_provider';  // 添加这一行
```

### 步骤 4：添加 UI 图标

更新 `src/components/account/ProviderIcon.tsx`：

```tsx
const providerIcons: Record<ProviderType, React.ReactNode> = {
  // ... 现有服务商
  your_provider: <YourProviderIcon className="w-5 h-5" />,
};
```

### 步骤 5：添加翻译

更新翻译文件：

**`src/i18n/locales/en-US.ts`：**
```typescript
providers: {
  // ... 现有服务商
  your_provider: 'Your Provider',
}
```

**`src/i18n/locales/zh-CN.ts`：**
```typescript
providers: {
  // ... 现有服务商
  your_provider: '你的服务商',
}
```

### 步骤 6：测试服务商

1. 启动开发服务器：`pnpm tauri dev`
2. 使用新服务商添加账号
3. 测试所有操作：列出域名、列出记录、创建/更新/删除记录
4. 验证分页和搜索功能

### 参考实现

完整示例请参阅：
- **简单服务商**：`src-tauri/src/providers/cloudflare.rs`
- **复杂服务商**：`src-tauri/src/providers/aliyun.rs`

## 构建与发布

### 本地构建

```bash
# 开发构建（更快，包含调试信息）
cargo build --manifest-path=src-tauri/Cargo.toml

# 生产构建（优化）
pnpm tauri build
```

### 版本管理

发布前：

1. 更新 `package.json` 中的版本号
2. 运行 `pnpm sync-version` 同步到其他文件
3. 提交更改：`git commit -am "chore: bump version to x.y.z"`
4. 创建 git 标签：`git tag vx.y.z`
5. 推送：`git push && git push --tags`

### GitHub Actions 发布

项目使用 GitHub Actions 进行自动化发布（`.github/workflows/release.yml`）。

**支持的平台：**
- macOS（Apple Silicon + Intel）
- Windows（x64 + ARM64）
- Linux（x64 + ARM64）

**触发发布：**

```bash
git tag v1.0.0
git push origin v1.0.0
```

工作流将：
1. 并行构建所有平台
2. 签名二进制文件（需要 `TAURI_SIGNING_PRIVATE_KEY` secret）
3. 创建 GitHub Release 草稿
4. 上传所有安装程序和更新清单

## 测试

### 运行测试

```bash
# 运行 Rust 测试
cargo test --manifest-path=src-tauri/Cargo.toml

# 运行前端测试（如果添加了测试）
pnpm test
```

### 手动测试清单

发布前，手动测试：

- [ ] 所有服务商的账号创建
- [ ] 凭证验证（有效和无效凭证）
- [ ] 域名列表与分页
- [ ] DNS 记录 CRUD 操作
- [ ] 搜索和过滤功能
- [ ] 带加密的账号导入导出
- [ ] DNS 查询工具
- [ ] WHOIS 查询工具
- [ ] 主题切换
- [ ] 语言切换
- [ ] 应用更新（如果配置了更新服务器）

## 常见问题

### 构建错误

**问题**：找不到 `webkit2gtk`（Linux）
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

**问题**：Rust 链接器错误
```bash
rustup update stable
cargo clean
```

**问题**：pnpm 安装失败
```bash
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

### 运行时错误

**问题**："加载凭证失败"
- 确保系统钥匙串服务正在运行（Linux：`gnome-keyring` 或 `kwallet`）

**问题**：开发中的 CORS 错误
- Tauri 应用使用自定义协议 `tauri://localhost`，绕过了 CORS

**问题**：服务商 API 错误
- 检查 API 凭证是否正确
- 验证 API 端点是否可访问（检查防火墙/代理）
- 启用调试日志：`RUST_LOG=debug pnpm tauri dev`

### 开发技巧

1. **使用 React DevTools**：检查 Zustand stores 和组件状态
2. **查看 Rust 日志**：后端错误在开发模式下会记录到控制台
3. **使用真实凭证测试**：尽可能使用测试/沙盒 API 密钥
4. **增量编译**：保持 `pnpm tauri dev` 运行以加快迭代速度
5. **遇到奇怪错误时清理构建**：`cargo clean && pnpm tauri dev`

## 获取帮助

- **文档**：[Tauri 文档](https://tauri.app/)、[React 文档](https://react.dev/)
- **问题**：[GitHub Issues](https://github.com/AptS-1547/dns-orchestrator/issues)
- **讨论**：[GitHub Discussions](https://github.com/AptS-1547/dns-orchestrator/discussions)

## 贡献

贡献指南请参阅主 README 中的[贡献部分](../README.zh-CN.md#贡献)。

---

祝编码愉快！🚀
