import i18n from "@/i18n"
import type { ApiError, DnsErrorCode, ProviderErrorDetails } from "@/types"

/**
 * 将 PascalCase 转换为 snake_case
 */
function toSnakeCase(str: string): string {
  return str.replace(/([A-Z])/g, "_$1").toLowerCase().replace(/^_/, "")
}

/**
 * 检查是否为 ProviderError
 */
function isProviderError(
  error: ApiError
): error is ApiError & { details: ProviderErrorDetails } {
  return (
    error.code === "Provider" &&
    typeof error.details === "object" &&
    error.details !== null &&
    "code" in error.details &&
    "provider" in error.details
  )
}

/**
 * 获取 ProviderError 的用户友好消息（带 fallback 链）
 *
 * Fallback 链:
 * 1. errors.provider.{provider}.{error_code} - Provider 特定翻译
 * 2. errors.provider.common.{error_code} - 通用翻译
 * 3. raw_message - 原始错误消息
 * 4. errors.unknown - 兜底
 */
function getProviderErrorMessage(details: ProviderErrorDetails): string {
  const provider = details.provider
  const errorCode = toSnakeCase(details.code)

  // 构建翻译参数
  const params: Record<string, unknown> = { ...details }

  // Fallback 链
  const keys = [
    `errors.provider.${provider}.${errorCode}`,
    `errors.provider.common.${errorCode}`,
  ]

  for (const key of keys) {
    if (i18n.exists(key)) {
      return i18n.t(key, params)
    }
  }

  // 从 details 中提取 raw_message 作为 fallback
  if ("raw_message" in details && details.raw_message) {
    return details.raw_message
  }

  if ("detail" in details && details.detail) {
    return details.detail
  }

  return i18n.t("errors.unknown")
}

/**
 * 从 ApiError 提取原始消息（用于非 Provider 错误）
 */
function extractRawMessage(error: ApiError | undefined): string {
  if (!error) return ""

  // ApiError 变体：details 是对象
  if (typeof error.details === "object" && error.details !== null) {
    if ("message" in error.details) {
      return error.details.message
    }
  }

  // 单值变体：details 是字符串
  if (typeof error.details === "string") {
    return error.details
  }

  // 无值变体：用 code 作为消息
  return error.code
}

/**
 * 解析错误码格式 "ERROR_CODE" 或 "ERROR_CODE: detail"
 */
function parseErrorCode(message: string): { code: string; detail?: string } | null {
  const match = message.match(/^([A-Z][A-Z0-9_]+)(?::\s*(.+))?$/)
  if (match) {
    return { code: match[1], detail: match[2] }
  }
  return null
}

/**
 * 从 ApiError 提取用户友好的错误消息（支持 i18n）
 */
export function getErrorMessage(error: ApiError | undefined): string {
  if (!error) return i18n.t("errors.unknown")

  // DEBUG: 打印收到的错误对象
  console.log("[getErrorMessage] error:", JSON.stringify(error, null, 2))

  // 处理 ProviderError
  if (isProviderError(error)) {
    console.log("[getErrorMessage] isProviderError: true, details:", JSON.stringify(error.details, null, 2))
    return getProviderErrorMessage(error.details)
  }

  console.log("[getErrorMessage] isProviderError: false")

  const raw = extractRawMessage(error)
  if (!raw) return i18n.t("errors.unknown")

  // 尝试解析错误码
  const parsed = parseErrorCode(raw)
  if (parsed) {
    const key = `errors.${parsed.code.toLowerCase()}`
    if (i18n.exists(key)) {
      return i18n.t(key, { detail: parsed.detail })
    }
  }

  // fallback: 返回原始消息
  return raw
}

/**
 * 检查是否为特定错误类型
 */
export function isErrorCode(error: ApiError | undefined, code: DnsErrorCode): boolean {
  return error?.code === code
}

/**
 * 从 catch 块的 unknown 错误中提取消息
 * 处理 Tauri 抛出的各种错误格式
 */
export function extractErrorMessage(err: unknown): string {
  // DEBUG
  console.log("[extractErrorMessage] err:", err)
  console.log("[extractErrorMessage] typeof err:", typeof err)

  if (!err) return i18n.t("errors.unknown")

  // 字符串直接返回
  if (typeof err === "string") return err

  // JavaScript Error 对象
  if (err instanceof Error) return err.message

  // 对象类型（可能是 ApiError / DnsError 序列化结果）
  if (typeof err === "object") {
    const obj = err as Record<string, unknown>

    // Tauri 序列化的 DnsError: { code: "...", details: ... }
    if ("code" in obj && typeof obj.code === "string") {
      return getErrorMessage(obj as unknown as ApiError)
    }

    // 普通对象有 message 属性
    if ("message" in obj && typeof obj.message === "string") {
      return obj.message
    }

    // fallback: 序列化对象
    return JSON.stringify(err)
  }

  return String(err)
}

/**
 * 获取 API 错误的 provider（仅 ApiError 类型有）
 */
export function getErrorProvider(error: ApiError | undefined): string | undefined {
  if (!error) return undefined

  // ProviderError
  if (isProviderError(error)) {
    return error.details.provider
  }

  // 旧的 ApiError 格式
  if (error.code === "ApiError" && typeof error.details === "object") {
    return (error.details as { provider?: string })?.provider
  }

  return undefined
}

/**
 * 检查是否为凭证错误（用于触发账户状态刷新）
 */
export function isCredentialError(err: unknown): boolean {
  if (!err || typeof err !== "object") return false

  const obj = err as Record<string, unknown>

  // 检查 DnsError::Provider(ProviderError::InvalidCredentials)
  if (obj.code === "Provider" && typeof obj.details === "object" && obj.details !== null) {
    const details = obj.details as Record<string, unknown>
    return details.code === "InvalidCredentials"
  }

  // 检查 DnsError::InvalidCredentials
  if (obj.code === "InvalidCredentials") {
    return true
  }

  return false
}
