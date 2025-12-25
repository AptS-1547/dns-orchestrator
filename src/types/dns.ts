/** DNS 记录类型枚举 */
export type DnsRecordType = "A" | "AAAA" | "CNAME" | "MX" | "TXT" | "NS" | "SRV" | "CAA"

/** 所有可用的记录类型列表 */
export const RECORD_TYPES: DnsRecordType[] = ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SRV", "CAA"]

/** DNS 记录数据 - 类型安全的多态表示 */
export type RecordData =
  | { type: "A"; content: { address: string } }
  | { type: "AAAA"; content: { address: string } }
  | { type: "CNAME"; content: { target: string } }
  | { type: "MX"; content: { priority: number; exchange: string } }
  | { type: "TXT"; content: { text: string } }
  | { type: "NS"; content: { nameserver: string } }
  | { type: "SRV"; content: { priority: number; weight: number; port: number; target: string } }
  | { type: "CAA"; content: { flags: number; tag: string; value: string } }

/** DNS 记录 */
export interface DnsRecord {
  id: string
  domainId: string
  name: string
  ttl: number
  data: RecordData
  proxied?: boolean
  createdAt?: string
  updatedAt?: string
}

/** 创建 DNS 记录请求 */
export interface CreateDnsRecordRequest {
  domainId: string
  name: string
  ttl: number
  data: RecordData
  proxied?: boolean
}

/** 更新 DNS 记录请求 */
export interface UpdateDnsRecordRequest {
  domainId: string
  name: string
  ttl: number
  data: RecordData
  proxied?: boolean
}

/** 批量删除请求 */
export interface BatchDeleteRequest {
  domainId: string
  recordIds: string[]
}

/** 批量删除结果 */
export interface BatchDeleteResult {
  successCount: number
  failedCount: number
  failures: BatchDeleteFailure[]
}

/** 批量删除失败项 */
export interface BatchDeleteFailure {
  recordId: string
  reason: string
}

/** 常用 TTL 选项 */
export const TTL_OPTIONS = [
  { value: 1, labelKey: "dns.ttlAuto" },
  { value: 60, labelKey: "dns.ttlMinute" },
  { value: 300, labelKey: "dns.ttlMinutes", count: 5 },
  { value: 600, labelKey: "dns.ttlMinutes", count: 10 },
  { value: 1800, labelKey: "dns.ttlMinutes", count: 30 },
  { value: 3600, labelKey: "dns.ttlHour" },
  { value: 7200, labelKey: "dns.ttlHours", count: 2 },
  { value: 18000, labelKey: "dns.ttlHours", count: 5 },
  { value: 43200, labelKey: "dns.ttlHours", count: 12 },
  { value: 86400, labelKey: "dns.ttlDay" },
] as const

/** 记录类型描述 */
export const RECORD_TYPE_INFO: Record<DnsRecordType, { descriptionKey: string; example: string }> =
  {
    A: { descriptionKey: "dns.recordTypes.A", example: "192.168.1.1" },
    AAAA: { descriptionKey: "dns.recordTypes.AAAA", example: "2001:db8::1" },
    CNAME: { descriptionKey: "dns.recordTypes.CNAME", example: "www.example.com" },
    MX: { descriptionKey: "dns.recordTypes.MX", example: "mail.example.com" },
    TXT: { descriptionKey: "dns.recordTypes.TXT", example: "v=spf1 include:..." },
    NS: { descriptionKey: "dns.recordTypes.NS", example: "ns1.example.com" },
    SRV: { descriptionKey: "dns.recordTypes.SRV", example: "sip.example.com" },
    CAA: { descriptionKey: "dns.recordTypes.CAA", example: "letsencrypt.org" },
  }
