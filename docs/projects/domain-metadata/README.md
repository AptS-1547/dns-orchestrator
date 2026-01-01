# 域名元数据系统项目

域名收藏、标签、备注等自定义元数据功能的实施计划和文档。

## 项目概述

为 DNS Orchestrator 添加用户自定义的域名元数据功能，支持：
- **收藏标记**：星标收藏域名
- **标签系统**：多标签分类管理
- **扩展元数据**：颜色标记、备注等（可扩展）
- **后端持久化**：使用 tauri-plugin-store，类似账户存储
- **类型安全**：完整的 Rust + TypeScript 类型系统

## 项目状态

🚧 **进行中** - Phase 1 基础收藏功能实施中

### 分阶段计划

| Phase | 功能 | 状态 |
|-------|------|------|
| Phase 1 | 基础收藏功能（MVP） | 🚧 进行中 |
| Phase 2 | 标签系统 | 📋 计划中 |
| Phase 3 | 完整元数据（颜色、备注） | 📋 计划中 |

## 文档导航

- **[实施指南](./implementation-guide.md)** - 详细的分步实施教程
- **[架构设计](./architecture.md)** - 系统架构和数据流说明
- **[API 参考](./api-reference.md)** - 前后端 API 文档

## 快速链接

### 核心代码位置

**后端（Rust）**：
- Core 层类型：`dns-orchestrator-core/src/types/domain_metadata.rs`
- Repository trait：`dns-orchestrator-core/src/traits/domain_metadata_repository.rs`
- Service：`dns-orchestrator-core/src/services/domain_metadata_service.rs`
- Adapter：`src-tauri/src/adapters/domain_metadata_repository.rs`
- Tauri 命令：`src-tauri/src/commands/domain_metadata.rs`

**前端（TypeScript）**：
- 类型定义：`src/types/domain-metadata.ts`
- Service：`src/services/domainMetadata.service.ts`
- Store：`src/stores/domainStore.ts`（扩展）
- UI 组件：`src/components/domain/DomainFavoriteButton.tsx`

## 相关 Issue

- [#31 feat: 能否给域名打上收藏](https://github.com/AptS-1547/dns-orchestrator/issues/31)

## 技术栈

- **后端持久化**：tauri-plugin-store（存储到 `domain_metadata.json`）
- **依赖注入**：trait + Arc<dyn Trait>
- **前端状态管理**：Zustand
- **类型安全**：Rust serde + TypeScript

---

**返回**: [项目文档目录](../README.md)
