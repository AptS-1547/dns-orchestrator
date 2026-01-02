# 架构设计

DNS Orchestrator 的系统架构设计文档。

## 📖 文档列表

### [架构概览](./overview.md)

整体架构设计：
- 四层架构模式
- 技术栈选择
- 跨平台支持
- 依赖注入模式
- Transport 抽象

### [Core 库设计](./core-library.md)

核心业务逻辑库：
- Service 层设计
- Repository 模式
- 错误处理
- 类型系统

### [Provider 抽象](./provider-abstraction.md)

DNS 服务商抽象层：
- DnsProvider Trait
- 工厂模式
- 凭证验证
- API 调用封装

### [架构图](./diagrams/)

可视化架构设计：
- 系统架构图
- 数据流图
- 组件关系图

## 🎯 设计原则

1. **平台无关**: Core 库不依赖任何平台特定实现
2. **依赖注入**: 通过 Trait 抽象平台特定功能
3. **类型安全**: 充分利用 Rust 的类型系统
4. **可测试性**: 所有组件都可以独立测试

## 🏗️ 四层架构

```
Frontend (React + Zustand)
    ↓ Transport Abstraction
Backend (Tauri Commands / Actix-web)
    ↓ ServiceContext (DI)
Core Library (dns-orchestrator-core)
    ↓ DnsProvider Trait
Provider Library (dns-orchestrator-provider)
    ↓ HTTPS
DNS Provider APIs
```

## 🔗 相关文档

- [开发文档](../development/) - 如何开发和贡献
- [API 参考](../api/) - API 使用说明
- [项目管理](../projects/) - 当前项目状态

---

**返回**: [文档中心](../README.md)
