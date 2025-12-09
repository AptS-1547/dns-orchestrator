/**
 * Tauri Transport 实现
 * 通过 Tauri IPC 调用 Rust 后端
 */

import { invoke as tauriInvoke } from "@tauri-apps/api/core"
import type { CommandMap, ITransport } from "./types"

class TauriTransport implements ITransport {
  invoke<K extends keyof CommandMap>(
    command: K,
    args?: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]> {
    return tauriInvoke<CommandMap[K]["result"]>(command, args ?? {})
  }
}

export const transport: ITransport = new TauriTransport()
