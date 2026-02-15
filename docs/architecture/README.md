# Architecture Documentation

DNS Orchestrator 的系统架构设计文档。

## 📖 文档列表

### [架构概览](./overview.md)

整体架构设计：
- 五层架构模式
- Crate 依赖关系
- 技术栈选择
- 跨平台支持
- 依赖注入模式
- Transport 抽象

### [App 引导层设计](./app-bootstrap.md)

多前端共享引导层：
- AppState 服务容器
- AppStateBuilder 适配器注入
- StartupHooks 启动回调
- 多前端接入方式

### [Core 库设计](./core-library.md)

核心业务逻辑库：
- ServiceContext 依赖注入
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
3. **共享引导**: 通过 App 层统一服务组装和启动流程
4. **类型安全**: 充分利用 Rust 的类型系统
5. **可测试性**: 所有组件都可以独立测试

## 🏗️ 五层架构

```
Frontend (React + Zustand)
    ↓ Transport Abstraction
Backend (Tauri Commands / Actix-web Handlers / TUI)
    ↓ AppStateBuilder
App Bootstrap (dns-orchestrator-app)
    ↓ ServiceContext (DI)
Core Library (dns-orchestrator-core)
    ↓ DnsProvider Trait
Provider Library (dns-orchestrator-provider)
    ↓ HTTPS
DNS Provider APIs
```

## 📦 Crate 依赖关系

```
dns-orchestrator-provider    (最底层，零内部依赖)
│
├──▶ dns-orchestrator-core    (业务逻辑层)
│
├──▶ dns-orchestrator-app     (引导层)
│
│   dns-orchestrator-toolbox  (独立工具库)
│
├──▶ dns-orchestrator-tauri   (Tauri 前端)
├──▶ dns-orchestrator-web     (Web 前端)
╰──▶ dns-orchestrator-tui     (TUI 前端，未来)
```

## 🔗 相关文档

- [开发文档](../development/) - 如何开发和贡献
- [API 参考](../api/) - API 使用说明
- [项目管理](../projects/) - 当前项目状态

---

**返回**: [文档中心](../README.md)
