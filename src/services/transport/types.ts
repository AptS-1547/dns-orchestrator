/**
 * Transport 层类型定义
 * 抽象 Tauri IPC 和 HTTP 调用的统一接口
 */

import type {
  Account,
  ApiResponse,
  BatchDeleteRequest,
  BatchDeleteResult,
  BatchTagRequest,
  BatchTagResult,
  CreateAccountRequest,
  CreateDnsRecordRequest,
  DnsLookupResult,
  DnsPropagationResult,
  DnsRecord,
  DnssecResult,
  Domain,
  DomainMetadata,
  DomainMetadataUpdate,
  ExportAccountsRequest,
  ExportAccountsResponse,
  HttpHeaderCheckRequest,
  HttpHeaderCheckResult,
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

// ============ Command 类型映射 ============

/** 所有 command 的类型映射 */
export interface CommandMap {
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
  batch_delete_accounts: {
    args: { accountIds: string[] }
    result: ApiResponse<BatchDeleteResult>
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
  is_restore_completed: {
    args: Record<string, never>
    result: boolean
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

  // Domain metadata commands
  get_domain_metadata: {
    args: { accountId: string; domainId: string }
    result: ApiResponse<DomainMetadata>
  }
  toggle_domain_favorite: {
    args: { accountId: string; domainId: string }
    result: ApiResponse<boolean>
  }
  list_account_favorite_domain_keys: {
    args: { accountId: string }
    result: ApiResponse<string[]>
  }
  add_domain_tag: {
    args: { accountId: string; domainId: string; tag: string }
    result: ApiResponse<string[]>
  }
  remove_domain_tag: {
    args: { accountId: string; domainId: string; tag: string }
    result: ApiResponse<string[]>
  }
  set_domain_tags: {
    args: { accountId: string; domainId: string; tags: string[] }
    result: ApiResponse<string[]>
  }
  batch_add_domain_tags: {
    args: { requests: BatchTagRequest[] }
    result: ApiResponse<BatchTagResult>
  }
  batch_remove_domain_tags: {
    args: { requests: BatchTagRequest[] }
    result: ApiResponse<BatchTagResult>
  }
  batch_set_domain_tags: {
    args: { requests: BatchTagRequest[] }
    result: ApiResponse<BatchTagResult>
  }
  find_domains_by_tag: {
    args: { tag: string }
    result: ApiResponse<string[]>
  }
  list_all_domain_tags: {
    args: Record<string, never>
    result: ApiResponse<string[]>
  }
  update_domain_metadata: {
    args: { accountId: string; domainId: string; update: DomainMetadataUpdate }
    result: ApiResponse<DomainMetadata>
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
  http_header_check: {
    args: { request: HttpHeaderCheckRequest }
    result: ApiResponse<HttpHeaderCheckResult>
  }
  dns_propagation_check: {
    args: { domain: string; recordType: string }
    result: ApiResponse<DnsPropagationResult>
  }
  dnssec_check: {
    args: { domain: string; nameserver: string | null }
    result: ApiResponse<DnssecResult>
  }
}

// ============ 类型工具 ============

/** 提取无参数的 command */
export type NoArgsCommands = {
  [K in keyof CommandMap]: CommandMap[K]["args"] extends Record<string, never> ? K : never
}[keyof CommandMap]

/** 提取有参数的 command */
export type WithArgsCommands = Exclude<keyof CommandMap, NoArgsCommands>

// ============ Transport 接口 ============

/** Transport 抽象接口 */
export interface ITransport {
  /** 无参数调用 */
  invoke<K extends NoArgsCommands>(command: K): Promise<CommandMap[K]["result"]>

  /** 有参数调用 */
  invoke<K extends WithArgsCommands>(
    command: K,
    args: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]>
}
