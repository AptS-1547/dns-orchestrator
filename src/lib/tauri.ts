/**
 * 类型安全的 Tauri invoke wrapper
 * 为所有 Tauri command 提供参数和返回值的类型约束
 */

import { type Channel, invoke as tauriInvoke } from "@tauri-apps/api/core"
import type {
  Account,
  ApiResponse,
  BatchDeleteRequest,
  BatchDeleteResult,
  CreateAccountRequest,
  CreateDnsRecordRequest,
  DnsLookupResult,
  DnsRecord,
  Domain,
  ExportAccountsRequest,
  ExportAccountsResponse,
  ImportAccountsRequest,
  ImportPreview,
  ImportResult,
  IpLookupResult,
  PaginatedResponse,
  ProviderInfo,
  SslCheckResult,
  UpdateDnsRecordRequest,
  WhoisResult,
} from "@/types"

// ============ Android 更新相关类型 ============

/** Android 更新信息 */
export interface AndroidUpdate {
  version: string
  notes: string
  url: string
}

/** 下载进度事件 */
export interface DownloadProgress {
  event: "Started" | "Progress" | "Finished"
  data: {
    content_length?: number
    chunk_length?: number
  }
}

// ============ Command 类型映射 ============

/** 所有 Tauri command 的类型映射 */
interface CommandMap {
  // Account commands
  list_accounts: {
    args: Record<string, never>
    result: ApiResponse<Account[]>
  }
  create_account: {
    args: { request: CreateAccountRequest }
    result: ApiResponse<Account>
  }
  delete_account: {
    args: { accountId: string }
    result: ApiResponse<void>
  }
  list_providers: {
    args: Record<string, never>
    result: ApiResponse<ProviderInfo[]>
  }
  export_accounts: {
    args: { request: ExportAccountsRequest }
    result: ApiResponse<ExportAccountsResponse>
  }
  preview_import: {
    args: { content: string; password: string | null }
    result: ApiResponse<ImportPreview>
  }
  import_accounts: {
    args: { request: ImportAccountsRequest }
    result: ApiResponse<ImportResult>
  }

  // Domain commands
  list_domains: {
    args: { accountId: string; page?: number; pageSize?: number }
    result: ApiResponse<PaginatedResponse<Domain>>
  }
  get_domain: {
    args: { accountId: string; domainId: string }
    result: ApiResponse<Domain>
  }

  // DNS commands
  list_dns_records: {
    args: {
      accountId: string
      domainId: string
      page?: number
      pageSize?: number
      keyword?: string | null
      recordType?: string | null
    }
    result: ApiResponse<PaginatedResponse<DnsRecord>>
  }
  create_dns_record: {
    args: { accountId: string; request: CreateDnsRecordRequest }
    result: ApiResponse<DnsRecord>
  }
  update_dns_record: {
    args: { accountId: string; recordId: string; request: UpdateDnsRecordRequest }
    result: ApiResponse<DnsRecord>
  }
  delete_dns_record: {
    args: { accountId: string; recordId: string; domainId: string }
    result: ApiResponse<void>
  }
  batch_delete_dns_records: {
    args: { accountId: string; request: BatchDeleteRequest }
    result: ApiResponse<BatchDeleteResult>
  }

  // Toolbox commands
  whois_lookup: {
    args: { domain: string }
    result: ApiResponse<WhoisResult>
  }
  dns_lookup: {
    args: { domain: string; recordType: string; nameserver: string | null }
    result: ApiResponse<DnsLookupResult>
  }
  ip_lookup: {
    args: { query: string }
    result: ApiResponse<IpLookupResult>
  }
  ssl_check: {
    args: { domain: string; port?: number }
    result: ApiResponse<SslCheckResult>
  }

  // Android updater commands
  check_android_update: {
    args: { currentVersion: string }
    result: AndroidUpdate | null
  }
  download_apk: {
    args: { url: string; onProgress: Channel<DownloadProgress> }
    result: string
  }
  install_apk: {
    args: { path: string }
    result: undefined
  }
}

// ============ 类型工具 ============

/** 提取无参数的 command */
type NoArgsCommands = {
  [K in keyof CommandMap]: CommandMap[K]["args"] extends Record<string, never> ? K : never
}[keyof CommandMap]

/** 提取有参数的 command */
type WithArgsCommands = Exclude<keyof CommandMap, NoArgsCommands>

// ============ 类型安全的 invoke 函数 ============

/**
 * 类型安全的 invoke - 无参数版本
 */
export function invoke<K extends NoArgsCommands>(command: K): Promise<CommandMap[K]["result"]>

/**
 * 类型安全的 invoke - 有参数版本
 */
export function invoke<K extends WithArgsCommands>(
  command: K,
  args: CommandMap[K]["args"]
): Promise<CommandMap[K]["result"]>

/**
 * invoke 实现
 */
export function invoke<K extends keyof CommandMap>(
  command: K,
  args?: CommandMap[K]["args"]
): Promise<CommandMap[K]["result"]> {
  return tauriInvoke<CommandMap[K]["result"]>(command, args ?? {})
}

// ============ 便利函数 ============

/** 预定义的便利函数，用于常用的无参数 command */
export const commands = {
  listAccounts: () => invoke("list_accounts"),
  listProviders: () => invoke("list_providers"),
} as const
