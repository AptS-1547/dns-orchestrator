# Changelog

本项目所有值得注意的变更都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
本项目遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [1.8.0] - 2026-01-10

### Added
- DNS 记录向导模式（网站、邮件、子域名、域名验证四大场景）
- 邮件服务商预设（Gmail、Microsoft 365、阿里云、腾讯企业邮箱）
- 标签选择下拉框组件（复用已有标签）
- 收藏域名页面标签筛选功能

### Changed
- 前端存储层统一重构（类型安全的 `IStorage` 接口）
- 域名列表组件拆分（DomainAccountGroup、DomainItem）
- 移动端批量操作栏、分页组件优化

## [1.7.0] - 2026-01-02

### Added
- 域名元数据管理系统（收藏、标签、颜色标记、备注）
- 批量标签操作（添加、移除、替换）
- DNS 记录实时提示功能
- 标签筛选器（Popover + Command 模式）
- 设置页面三标签页布局（外观、功能、关于）

### Changed
- 凭证存储类型安全重构（`HashMap<String, String>` → `ProviderCredentials` 枚举）
- CredentialStore trait 方法重命名：`load()` → `get()`、`save()` → `set()`、`delete()` → `remove()`
- 完整的文档体系重构

### Important
- **首次启动时会自动迁移凭证数据**
- **未来 v2.0.0 将不支持从 v1.6.x 直接升级，必须先升级到 v1.7.x**

## [1.6.0] - 2026-01-01

### Added
- HTTP 头检查工具（安全头分析、原始报文查看）
- DNS 传播检查工具（12 个全球节点）
- DNSSEC 验证工具（DNSKEY、DS、RRSIG 记录）
- 移动端工具箱折叠面板布局

### Changed
- SSL 检查底层从 `native-tls` 迁移到 `rustls`
- 新增 `useCopyToClipboard`、`useEnterKeyHandler`、`useToolboxQuery` Hooks
- 历史记录管理增强（确认对话框、按类型清除）

## [1.5.2] - 2025-12-27

### Fixed
- 扩展阿里云 DNS 错误码映射（约 40 个）
- 扩展 Cloudflare 错误码映射（约 15 个）

## [1.5.1] - 2025-12-27

### Added
- DNS 记录 @ 记录完整域名显示
- Provider 集成测试框架（Cloudflare、阿里云、DNSPod、华为云）
- 完整架构文档（中英文双语）

### Fixed
- 华为云 API 错误响应字段映射（`error_code` → `code`）

## [1.5.0] - 2025-12-25

### Changed
- **DNS 记录数据结构重构**：从扁平字符串改为类型安全的 tagged union
- MX/SRV/CAA 记录结构化表单编辑
- 四个 DNS Provider 完成数据结构适配

### Added
- `UnsupportedRecordType` 错误类型

## [1.4.2] - 2025-12-19

### Added
- 6 个通用布局组件（PageLayout、PageHeader、PageContainer、EmptyState、SectionCard、SettingSection）

### Fixed
- 系统主题跟随实时监听

### Changed
- 全局禁用文字选择（表单元素例外）

## [1.4.1] - 2025-12-18

### Added
- 批量 DNS 记录操作框架（接口定义，占位实现）
- 华为云错误码映射扩展（29 个核心错误码）

### Changed
- 日志级别按构建模式分离（Debug: debug, Release: warn）

### Security
- 依赖安全更新（asn1-rs、x509-parser、rand、sea-orm 等）

## [1.4.0] - 2025-12-15

### Added
- DNS 记录列表分页模式选择（无限滚动/传统分页）
- HTTP 请求自动重试机制（指数退避）
- Provider 分页限制信息透明化
- 账户批量删除和信息更新功能

### Changed
- HTTP Client 统一重构
- Provider 元数据自包含（添加新 Provider 更简单）
- 账户服务层拆分为 4 个专门服务

## [1.3.2] - 2025-12-15

### Fixed
- 阿里云 `domain_id` 不一致问题
- 移除阿里云 `list_records` 和 `create_record` 中多余的 API 调用

## [1.3.1] - 2025-12-13

### Changed
- 账户仓库缓存优化（`Arc<Vec<Account>>`）
- 凭证存储添加内存缓存层
- HTTP Client 改为全局共享单例
- 异步账户恢复（不阻塞启动）
- 凭证失效自动检测

### Fixed
- DNSPod `dns_status` 字段支持
- HomePage `recentDomains` 状态初始化

## [1.3.0] - 2025-12-13

### Added
- `dns-orchestrator-core` 平台无关核心库
- 存储层抽象（AccountRepository、CredentialStore、ProviderRegistry）

### Changed
- DNS Provider 模块化重构（拆分为 mod.rs、provider.rs、http.rs 等）
- 新增 `DomainLocked`、`PermissionDenied` 错误类型
- UI 状态迁移到 domainStore（expandedAccounts、scrollPosition）

## [1.2.0] - 2025-12-12

### Added
- 移动端汉堡菜单导航
- 账户删除自动清理相关数据
- Web 端文件操作和更新检查支持

### Changed
- React Router v7 迁移
- 文件服务抽象（统一 Tauri/Web 平台）
- 加载状态防重复提交

## [1.1.0] - 2025-12-11

### Added
- `dns-orchestrator-provider` 独立库
- Actix-web 后端项目（`src-actix-web`）
- 前端 Transport 抽象（ITransport 接口）
- Feature Flags 按需启用 Provider

### Changed
- 错误处理系统重构（ProviderError 统一错误映射）
- 前端服务层新增（`src/services/`）
- Rust Edition 2024

## [1.0.12] - 2025-12-10

### Added
- 首页仪表盘（域名总数、最近访问）
- 域名缓存机制和分页加载
- 多页面路由架构

### Changed
- 统一错误类型（DnsError 枚举）
- 类型安全的 Tauri invoke 包装器

## [1.0.11] - 2025-12-08

### Added
- 独立账户管理页面
- 侧边栏树形结构导航
- DNS 查询自定义服务器
- 移动端批量选择功能

### Changed
- IP 查询切换至 ipwho.is（支持 HTTPS）

## [1.0.10] - 2025-12-08

### Changed
- 全新设计的应用图标

## [1.0.9] - 2025-12-08

### Added
- SSL 证书检查工具
- IP 地理位置查询工具
- DNS 记录批量删除功能
- Android APK 安装器插件

## [1.0.8] - 2025-12-06

### Added
- 移动端 DNS 记录卡片布局
- 更新对话框（显示版本和发行说明）

### Fixed
- 屏幕宽度断点切换时页面状态丢失

## [1.0.7] - 2025-12-06

### Added
- Android 平台支持完善
- Android 应用内更新功能

## [1.0.6] - 2025-12-05

### Added
- DNS 记录分页、搜索和无限滚动功能

## [1.0.5] - 2025-12-03

### Added
- 账号导入导出功能

## [1.0.4] - 2025-12-03

### Added
- 华为云 DNS Provider 支持

## [1.0.3] - 2025-12-02

### Changed
- 版本号更新和内部优化

## [1.0.2] - 2025-12-02

### Changed
- 设置页面重构
- 添加状态栏

## [1.0.1] - 2025-12-02

### Added
- macOS Intel 架构支持

### Changed
- 自动更新功能增强

## [1.0.0] - 2025-12-02

### Added
- 首个正式版本发布
- 自动更新功能
- 支持 Cloudflare、阿里云、DNSPod 三个 DNS Provider
- DNS 记录 CRUD 操作
- 跨平台支持（macOS、Windows、Linux、Android）

[1.8.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.7.0...v1.8.0
[1.7.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.5.2...v1.6.0
[1.5.2]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.5.1...v1.5.2
[1.5.1]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.3.2...v1.4.0
[1.3.2]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.3.1...v1.3.2
[1.3.1]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.3.0...v1.3.1
[1.3.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.12...v1.1.0
[1.0.12]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.11...v1.0.12
[1.0.11]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.10...v1.0.11
[1.0.10]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.9...v1.0.10
[1.0.9]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.8...v1.0.9
[1.0.8]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.7...v1.0.8
[1.0.7]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.6...v1.0.7
[1.0.6]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.5...v1.0.6
[1.0.5]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.4...v1.0.5
[1.0.4]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.2...v1.0.3
[1.0.2]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.1...v1.0.2
[1.0.1]: https://github.com/AptS-1547/dns-orchestrator/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/AptS-1547/dns-orchestrator/releases/tag/v1.0.0
