/**
 * Tauri 相关类型定义
 *
 * 注意：invoke 函数已迁移到 /src/services/transport/
 * 请使用 /src/services/ 中的 Service 层进行调用
 *
 * @deprecated 此文件仅保留类型导出以保持向后兼容，请使用 @/services/transport/types
 */

// Re-export types from transport layer
export type {
  AndroidUpdate,
  CommandMap,
  DownloadProgress,
  NoArgsCommands,
  WithArgsCommands,
} from "@/services/transport/types"
