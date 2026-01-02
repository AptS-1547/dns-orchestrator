# CredentialStore Trait 重构项目

**Issue**: [#28](https://github.com/AptS-1547/dns-orchestrator/issues/28)
**版本**: v1.7.0
**状态**: ✅ 已完成
**日期**: 2026-01-01

## 项目概述

将 CredentialStore 从弱类型 `HashMap<String, HashMap<String, String>>` 重构为类型安全的 `HashMap<String, ProviderCredentials>` 枚举，提升类型安全性和代码可维护性。

## 核心改进

1. **类型安全**: 使用 Rust 枚举确保编译时类型检查
2. **自动迁移**: 支持从 v1.6.x 旧格式自动升级
3. **方法重命名**: 遵循 Rust 命名规范（`load` → `get`, `save` → `set`, `delete` → `remove`）
4. **向后兼容**: v1.7.0 支持读取旧格式并自动迁移

## 文档导航

- [实施细节](./implementation.md) - 完整的实施记录和修改的文件清单（v1.7.0）
- [v2.0.0 发布准备](./v2-preparation.md) - v2.0.0 发布前的完整准备清单
- [v2.0.0 清理计划](./v2-cleanup.md) - 移除兼容代码的技术细节

## 相关文件

### Core 库（10 个）

1. `dns-orchestrator-core/src/traits/credential_store.rs`
2. `dns-orchestrator-core/src/traits/mod.rs`
3. `dns-orchestrator-core/src/services/migration_service.rs` (新建)
4. `dns-orchestrator-core/src/services/mod.rs`
5. `dns-orchestrator-core/src/services/credential_management_service.rs`
6. `dns-orchestrator-core/src/services/account_lifecycle_service.rs`
7. `dns-orchestrator-core/src/services/account_bootstrap_service.rs`
8. `dns-orchestrator-core/src/services/import_export_service.rs`
9. `dns-orchestrator-core/src/error.rs`
10. `dns-orchestrator-core/src/types/account.rs`

### Tauri 后端（3 个）

11. `src-tauri/src/adapters/credential_store.rs`
12. `src-tauri/src/error.rs`
13. `src-tauri/src/lib.rs`
14. `src-tauri/src/types.rs`

### 前端（3 个）

15. `src/types/account.ts`
16. `src/components/account/AccountForm.tsx`
17. `docs/TODO.md`

## 升级路径

```
v1.6.x → v1.7.x: 自动迁移 ✅
v1.7.x → v2.0.x: 无需迁移 ✅
v1.6.x → v2.0.x: ⚠️ 不支持，必须先升级到 v1.7.x
```

## 关键决策

1. **数据格式版本化**: 使用 `#[serde(untagged)]` 自动检测 V1/V2 格式
2. **迁移时机**: 启动时自动执行（使用 `block_on` 确保完成）
3. **失败处理**: 记录失败账户但不中断启动，允许用户手动修复
4. **前端重构**: 同步更新前端类型定义，保持一致性

---

**最后更新**: 2026-01-01
