# DNS Orchestrator

![GitHub release (latest by date)](https://img.shields.io/github/v/release/AptS-1547/dns-orchestrator)
![GitHub Downloads](https://img.shields.io/github/downloads/AptS-1547/dns-orchestrator/total)
![Release Workflow](https://github.com/AptS-1547/dns-orchestrator/actions/workflows/release.yml/badge.svg)
![License](https://img.shields.io/github/license/AptS-1547/dns-orchestrator)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20Android-blue)

跨平台 DNS 管理项目，统一管理多个 DNS 服务商的域名解析记录。

简体中文 | [English](./README.md)

## 功能特性

- **多账号管理** - 支持管理多个 DNS 服务商账号，凭证安全存储
- **统一 DNS 管理** - 跨服务商创建、查看、更新和删除 DNS 记录
- **高级搜索与过滤** - 分页、实时搜索、记录类型过滤，支持无限滚动
- **账号导入导出** - 加密备份和迁移账号配置
- **网络工具箱** - 内置 DNS 查询、WHOIS、IP 地理位置、SSL 证书、HTTP Header、DNS 传播和 DNSSEC 工具
- **跨平台支持** - 原生体验，支持 macOS、Windows、Linux 和 Android
- **现代化界面** - 简洁的用户界面，支持深色/浅色主题和中英文双语切换

## 支持的 DNS 服务商

| 服务商 | 特性 |
|--------|------|
| **Cloudflare** | 完整的 DNS 管理，支持 CDN 代理开关 |
| **阿里云 DNS** | 全面的记录管理，支持分页和过滤 |
| **腾讯云 DNSPod** | 完整的 DNS 操作，支持搜索功能 |
| **华为云 DNS** | 全功能 DNS 管理，支持类型过滤 |

> 💡 **更多服务商即将支持！** 如果你需要支持特定的 DNS 服务商，欢迎[提交 issue](https://github.com/AptS-1547/dns-orchestrator/issues)。

## 快速开始

### 下载

从 [Releases](https://github.com/AptS-1547/dns-orchestrator/releases) 页面下载适合您平台的最新版本：

- **macOS**: `.dmg` (Apple Silicon / Intel)
- **Windows**: `.msi` 或 `.exe` (x64, ARM64)
- **Linux**: `.deb` 或 `.AppImage` (x64, ARM64)
- **Android**: `.apk` (ARM64, ARM32, x64)

### 安装

#### macOS
1. 下载适合您架构的 `.dmg` 文件
2. 打开 `.dmg` 并将 DNS Orchestrator 拖到应用程序文件夹
3. 从应用程序启动（首次运行可能需要在系统偏好设置 → 安全性与隐私中批准）

#### Windows
1. 下载 `.msi` 安装程序
2. 运行安装程序并按照安装向导操作
3. 从开始菜单启动 DNS Orchestrator

#### Linux
1. 下载 `.deb` 包或 `.AppImage`
2. 对于 `.deb`：`sudo dpkg -i dns-orchestrator_*.deb`
3. 对于 `.AppImage`：添加可执行权限（`chmod +x`）后直接运行

### 首次使用

1. 点击**"添加账号"**配置您的第一个 DNS 服务商
2. 选择服务商类型并输入 API 凭证
3. 查看和管理您的域名及 DNS 记录
4. 使用**网络工具箱**进行网络诊断（DNS、WHOIS、IP、SSL、HTTP Header、传播检查、DNSSEC）

## 核心功能

### 账号管理
- 添加任意数量的多服务商账号
- 使用系统钥匙串安全存储凭证（macOS Keychain、Windows 凭据管理器、Linux Secret Service）
- 带加密的账号导入导出功能，方便备份和迁移
- 账号间快速切换

### 域名管理
- 分页浏览所有服务商的域名
- 支持大量域名列表的无限滚动
- 快速域名选择和过滤

### DNS 记录管理
- **支持的记录类型**：A、AAAA、CNAME、MX、TXT、NS、SRV、CAA
- **分页加载**：每页高效加载 20 条记录
- **实时搜索**：带防抖的即时过滤
- **类型过滤**：按记录类型筛选，专注管理
- **批量操作**：带验证的创建、更新和删除记录
- **Cloudflare CDN 代理**：为 A/AAAA/CNAME 记录切换代理状态

### 网络工具箱
- **DNS 查询**：查询 DNS 记录（A、AAAA、CNAME、MX、TXT、NS、SOA、SRV、CAA、PTR、ALL）
- **WHOIS 查询**：检索域名注册信息
- **IP 地理位置**：查询 IP 或域名对应的国家/地区/城市/ISP/ASN
- **SSL 证书检查**：查看证书有效期、SAN、签发者和过期时间
- **HTTP Header 分析**：评估响应安全头并给出建议
- **DNS 传播检查**：对比全球解析器返回的一致性
- **DNSSEC 验证**：校验 DNSKEY/DS/RRSIG 部署状态
- **历史记录**：快速访问最近的查询

### 主题与本地化
- **主题**：浅色和深色模式，支持系统偏好设置自动检测
- **语言**：英文和简体中文
- 无需重启即可切换语言

## 技术栈

### 前端
- **框架**：React 19 + TypeScript 5
- **UI**：Tailwind CSS 4 + Radix UI 组件
- **状态管理**：Zustand 5
- **构建工具**：Vite 7
- **国际化**：i18next + react-i18next
- **图标**：Lucide React

### 后端
- **框架**：Tauri 2 + Rust workspace crates
- **核心逻辑**：`dns-orchestrator-core`
- **Provider 抽象**：`dns-orchestrator-provider`
- **网络诊断**：`dns-orchestrator-toolbox`
- **运行时**：Tokio（异步运行时）
- **HTTP 客户端**：Reqwest
- **凭证存储**：keyring 3（系统钥匙串集成）
- **加密**：内置的 crypto 模块用于账号导出

### 安全性
- API 凭证存储在系统钥匙串中，绝不以明文形式存储
- 带密码保护的加密账号导入导出
- 与 DNS 服务商之间的安全 HTTPS 通信

## 开发

### 前置要求
- Node.js 22+ 和 pnpm 10+
- Rust（最新稳定版）
- 平台特定依赖：
  - **macOS**：Xcode Command Line Tools
  - **Windows**：MSVC（Visual Studio 构建工具）
  - **Linux**：webkit2gtk、libappindicator、librsvg、patchelf

### 设置

```bash
# 克隆仓库
git clone https://github.com/AptS-1547/dns-orchestrator.git
cd dns-orchestrator

# 安装依赖
pnpm install

# 开发模式
pnpm tauri dev

# Web 开发模式
pnpm dev:web

# 生产构建
pnpm tauri build

# Web 构建
pnpm build:web

# 同步版本号（package.json → tauri.conf.json + Cargo.toml）
pnpm sync-version
```

详细的开发说明请参阅 [docs/development/README.md](./docs/development/README.md)。

## 架构

DNS Orchestrator 遵循清晰的架构模式：

- **前端层**：`dns-orchestrator-app`（React + Zustand）
- **平台层**：`dns-orchestrator-tauri` 与 `dns-orchestrator-web`
- **核心层**：`dns-orchestrator-core` 负责账户/域名/DNS 编排
- **Provider 层**：`dns-orchestrator-provider` 负责服务商集成
- **工具层**：`dns-orchestrator-toolbox` 负责网络诊断能力

深入的架构细节请参阅 [docs/architecture/README.md](./docs/architecture/README.md)。

## 系统要求

- **macOS**：10.13 (High Sierra) 或更高版本
- **Windows**：10 或更高版本
- **Linux**：带有 DBus Secret Service 的现代发行版（GNOME Keyring、KWallet 等）
- **Android**：7.0 (Nougat) 或更高版本

## 贡献

欢迎贡献！以下是您可以提供帮助的方式：

1. **报告 Bug**：提交包含复现步骤的 issue
2. **建议功能**：在 issues 中分享您的想法
3. **添加 DNS 服务商**：参考 [docs/development/README.md](./docs/development/README.md) 中的开发指南
4. **改进翻译**：更新 `src/i18n/locales/` 中的语言文件
5. **提交 Pull Request**：Fork、创建分支、编码、提交 PR

请确保您的代码遵循现有风格并包含适当的错误处理。

## 许可证

MIT License - 详见 [LICENSE](./LICENSE)。

## 致谢

使用 [Tauri](https://tauri.app/)、[React](https://react.dev/) 和 [Rust](https://www.rust-lang.org/) 构建。

---

**作者**：AptS:1547 (Yuhan Bian / 卞雨涵)
**仓库**：[github.com/AptS-1547/dns-orchestrator](https://github.com/AptS-1547/dns-orchestrator)
