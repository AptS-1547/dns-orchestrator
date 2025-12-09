/**
 * Transport 层统一导出
 * 通过 Vite alias 在编译时切换具体实现
 */

export type {
  AndroidUpdate,
  CommandMap,
  DownloadProgress,
  ITransport,
  NoArgsCommands,
  WithArgsCommands,
} from "./types"

// 通过 vite.config.ts 中的 alias 配置，编译时决定导入 tauri 还是 http 实现
// @ts-expect-error - 路径由 Vite alias 动态解析
export { transport } from "#transport-impl"
