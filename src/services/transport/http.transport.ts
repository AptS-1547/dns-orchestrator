/**
 * HTTP Transport 实现
 * 通过 HTTP 调用 actix_web 后端（类 RPC 风格）
 */

import type { CommandMap, ITransport } from "./types"

export interface HttpTransportConfig {
  baseUrl?: string
  timeout?: number
}

class HttpTransport implements ITransport {
  private baseUrl: string
  private timeout: number

  constructor(config?: HttpTransportConfig) {
    this.baseUrl = config?.baseUrl ?? import.meta.env.VITE_API_BASE_URL ?? "/api"
    this.timeout = config?.timeout ?? 30000
  }

  async invoke<K extends keyof CommandMap>(
    command: K,
    args?: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]> {
    const controller = new AbortController()
    const timeoutId = setTimeout(() => controller.abort(), this.timeout)

    try {
      const response = await fetch(`${this.baseUrl}/invoke`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ command, args: args ?? {} }),
        signal: controller.signal,
      })

      if (!response.ok) {
        // 尝试解析错误响应
        const errorBody = await response.json().catch(() => null)
        if (errorBody) {
          return errorBody as CommandMap[K]["result"]
        }
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      return response.json()
    } finally {
      clearTimeout(timeoutId)
    }
  }
}

export const transport: ITransport = new HttpTransport()
