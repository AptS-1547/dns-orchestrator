export * from "./account"
export * from "./dns"
export * from "./domain"
export * from "./provider"
export * from "./toolbox"

/** 通用 API 响应 */
export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: ApiError
}

/** 后端错误码 */
export type DnsErrorCode =
  | "ProviderNotFound"
  | "AccountNotFound"
  | "DomainNotFound"
  | "RecordNotFound"
  | "CredentialError"
  | "ApiError"
  | "InvalidCredentials"
  | "SerializationError"
  | "ValidationError"
  | "ImportExportError"
  | "Provider" // ProviderError 变体

/** Provider 错误码 */
export type ProviderErrorCode =
  | "NetworkError"
  | "InvalidCredentials"
  | "RecordExists"
  | "RecordNotFound"
  | "InvalidParameter"
  | "QuotaExceeded"
  | "DomainNotFound"
  | "ParseError"
  | "Unknown"

/** Provider 错误详情（根据 code 不同，结构不同） */
export type ProviderErrorDetails =
  | { code: "NetworkError"; provider: string; detail: string }
  | { code: "InvalidCredentials"; provider: string }
  | {
      code: "RecordExists"
      provider: string
      record_name: string
      raw_message?: string
    }
  | {
      code: "RecordNotFound"
      provider: string
      record_id: string
      raw_message?: string
    }
  | {
      code: "InvalidParameter"
      provider: string
      param: string
      detail: string
    }
  | { code: "QuotaExceeded"; provider: string; raw_message?: string }
  | { code: "DomainNotFound"; provider: string; domain: string }
  | { code: "ParseError"; provider: string; detail: string }
  | {
      code: "Unknown"
      provider: string
      raw_code?: string
      raw_message: string
    }

/** API 错误（匹配后端 DnsError 序列化格式） */
export interface ApiError {
  code: DnsErrorCode
  details?:
    | string
    | { provider: string; message: string }
    | ProviderErrorDetails
}

/** 分页参数 */
export interface PaginationParams {
  page: number
  pageSize: number
}

/** 分页响应 */
export interface PaginatedResponse<T> {
  items: T[]
  page: number
  pageSize: number
  totalCount: number
  hasMore: boolean
}
