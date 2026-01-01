# API 参考

DNS Orchestrator 的 API 文档，面向集成开发者。

## 📖 文档列表

### [Tauri 命令](./tauri-commands.md)

前端调用的所有 Tauri 命令：
- 账户管理命令
- 域名管理命令
- DNS 记录命令
- 工具箱命令
- 导入导出命令

### [Core API](./core-api.md)

Core 库的公共 API：
- Service 层 API
- Repository Trait
- 错误类型
- 核心数据类型

### [Provider API](./provider-api.md)

Provider 库的接口定义：
- DnsProvider Trait
- ProviderCredentials
- DNS 记录类型
- 分页和查询

## 🎯 适合人群

- 需要集成 DNS Orchestrator 的开发者
- 希望了解 API 设计的开发者
- 开发第三方工具的开发者

## 📝 API 设计原则

1. **类型安全**: 充分利用 Rust 和 TypeScript 的类型系统
2. **一致性**: 所有 API 遵循统一的命名和结构
3. **错误处理**: 明确的错误类型和错误信息
4. **向后兼容**: 尽量保持 API 稳定性

## 🔗 相关文档

- [架构设计](../architecture/) - 了解系统设计
- [开发文档](../development/) - 如何开发和贡献
- [用户指南](../guides/) - 功能使用说明

---

**返回**: [文档中心](../README.md)
