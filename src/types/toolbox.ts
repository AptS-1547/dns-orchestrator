/** WHOIS 查询结果 */
export interface WhoisResult {
  domain: string
  registrar?: string
  creationDate?: string
  expirationDate?: string
  updatedDate?: string
  nameServers: string[]
  status: string[]
  raw: string
}

/** DNS 查询记录 */
export interface DnsLookupRecord {
  recordType: string
  name: string
  value: string
  ttl: number
  priority?: number
}

/** 查询历史项 */
export interface QueryHistoryItem {
  id: string
  type: "whois" | "dns"
  query: string
  recordType?: string
  timestamp: number
}

/** DNS 查询支持的记录类型 */
export const DNS_RECORD_TYPES = [
  "A",
  "AAAA",
  "CNAME",
  "MX",
  "TXT",
  "NS",
  "SOA",
  "SRV",
  "CAA",
  "PTR",
  "ALL",
] as const

export type DnsLookupType = (typeof DNS_RECORD_TYPES)[number]
