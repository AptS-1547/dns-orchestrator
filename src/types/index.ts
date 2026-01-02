export * from "./account"
export * from "./dns"
export * from "./domain"
export * from "./domain-metadata"
export * from "./navigation"
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
  | "CredentialValidation" // 凭证验证错误（结构化）
  | "ApiError"
  | "InvalidCredentials"
  | "SerializationError"
  | "ValidationError"
  | "ImportExportError"
  | "NoAccountsSelected" // 导出时没有选中账号
  | "UnsupportedFileVersion" // 导入时版本不支持
  | "Provider" // ProviderError 变体

/** 凭证验证错误详情 */
export type CredentialValidationDetails =
  | { type: "missingField"; provider: string; field: string; label: string }
  | { type: "emptyField"; provider: string; field: string; label: string }
  | { type: "invalidFormat"; provider: string; field: string; label: string; reason: string }

/** Provider 错误码 */
export type ProviderErrorCode =
  | "NetworkError"
  | "InvalidCredentials"
  | "RecordExists"
  | "RecordNotFound"
  | "InvalidParameter"
  | "QuotaExceeded"
  | "DomainNotFound"
  | "DomainLocked"
  | "PermissionDenied"
  | "ParseError"
  | "SerializationError"
  | "Unknown"

/** Provider 错误详情（根据 code 不同，结构不同） */
export type ProviderErrorDetails =
  | { code: "NetworkError"; provider: string; detail: string }
  | { code: "InvalidCredentials"; provider: string; raw_message?: string }
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
  | { code: "DomainNotFound"; provider: string; domain: string; raw_message?: string }
  | { code: "DomainLocked"; provider: string; domain: string; raw_message?: string }
  | { code: "PermissionDenied"; provider: string; raw_message?: string }
  | { code: "ParseError"; provider: string; detail: string }
  | { code: "SerializationError"; provider: string; detail: string }
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
    | CredentialValidationDetails
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
