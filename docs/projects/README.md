# 项目管理

DNS Orchestrator 的内部项目跟踪和规划。

## 🗂️ 活跃项目

### 1. [Toolbox 功能扩展](./toolbox/)

**状态**: 🔄 进行中
**优先级**: 高

网络诊断工具箱的功能扩展项目。

**已完成功能**:
- DNS 查询
- WHOIS 查询
- SSL 证书检查
- IP 地理位置查询
- HTTP 头检查器
- DNS 传播检查
- DNSSEC 验证

**计划功能**:
- 端口扫描器
- Ping / 连通性测试
- 子域名发现
- Traceroute
- 批量 DNS 查询

### 2. [域名元数据系统](./domain-metadata/)

**状态**: 🔄 进行中
**优先级**: 中
**Issue**: [#31](https://github.com/AptS-1547/dns-orchestrator/issues/31)

为域名添加用户自定义元数据功能（收藏、标签、备注等）。

**Phase 1（进行中）**:
- 基础收藏功能
- 后端持久化（tauri-plugin-store）
- 星标按钮 UI

**Phase 2（计划中）**:
- 标签系统
- 标签筛选和管理

**Phase 3（计划中）**:
- 颜色标记
- 备注编辑
- 元数据编辑面板

### 3. [CredentialStore Trait 重构](./credentialstore-refactor/)

**状态**: ✅ 已完成（v1.7.0）
**Issue**: [#28](https://github.com/AptS-1547/dns-orchestrator/issues/28)

将凭证存储从弱类型升级为类型安全的枚举。

**核心改进**:
- 类型安全：`HashMap<String, String>` → `ProviderCredentials` 枚举
- 方法重命名：`load()` → `get()`, `save()` → `set()`
- 自动迁移：支持从 v1.6.x 旧格式自动升级

**下一步**: [v2.0.0 发布准备](./credentialstore-refactor/v2-preparation.md)

## 📊 项目状态定义

- 🔄 **进行中**: 正在开发或设计
- ✅ **已完成**: 已实现并发布
- 📝 **计划中**: 已列入规划，待评估
- ⏸️ **暂停**: 暂时搁置
- ❌ **已取消**: 不再实施

## 📝 如何添加新项目

1. 在 `docs/projects/` 下创建项目文件夹
2. 创建项目 `README.md`（项目概述）
3. 根据需要创建详细文档（实施细节、设计文档等）
4. 在本文件中添加项目条目

## 🔗 相关文档

- [开发文档](../development/) - 如何开发和贡献
- [架构设计](../architecture/) - 系统架构设计
- [发行说明](../release-notes/) - 各版本更新日志

---

**返回**: [文档中心](../README.md)

**最后更新**: 2026-01-01
