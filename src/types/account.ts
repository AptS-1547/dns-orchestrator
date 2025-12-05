/** 账号状态 */
export type AccountStatus = "active" | "error"

/** 账号基础信息 */
export interface Account {
  id: string
  name: string
  provider: string
  createdAt: string
  updatedAt: string
  status?: AccountStatus
  error?: string
}

/** 创建账号请求 */
export interface CreateAccountRequest {
  name: string
  provider: string
  credentials: Record<string, string>
}

/** 更新账号请求 */
export interface UpdateAccountRequest {
  id: string
  name?: string
  credentials?: Record<string, string>
}

// ============ 导入导出相关类型 ============

/** 导出请求 */
export interface ExportAccountsRequest {
  accountIds: string[]
  encrypt: boolean
  password?: string
}

/** 导出响应 */
export interface ExportAccountsResponse {
  content: string
  suggestedFilename: string
}

/** 导入请求 */
export interface ImportAccountsRequest {
  content: string
  password?: string
}

/** 导入预览 */
export interface ImportPreview {
  encrypted: boolean
  accountCount: number
  accounts?: ImportPreviewAccount[]
}

/** 导入预览账号 */
export interface ImportPreviewAccount {
  name: string
  provider: string
  hasConflict: boolean
}

/** 导入结果 */
export interface ImportResult {
  successCount: number
  failures: ImportFailure[]
}

/** 导入失败项 */
export interface ImportFailure {
  name: string
  reason: string
}
