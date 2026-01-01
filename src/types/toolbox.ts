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

/** DNS 查询结果（包含 nameserver 信息） */
export interface DnsLookupResult {
  /** 使用的 DNS 服务器 */
  nameserver: string
  /** 查询记录列表 */
  records: DnsLookupRecord[]
}

/** IP 地理位置信息 */
export interface IpGeoInfo {
  ip: string
  /** IP 版本: "IPv4" 或 "IPv6" */
  ipVersion: string
  country?: string
  countryCode?: string
  region?: string
  city?: string
  latitude?: number
  longitude?: number
  timezone?: string
  isp?: string
  org?: string
  asn?: string
  asName?: string
}

/** IP 查询结果（支持域名解析多个 IP） */
export interface IpLookupResult {
  /** 查询的原始输入（IP 或域名） */
  query: string
  /** 是否为域名查询 */
  isDomain: boolean
  /** IP 地理位置结果列表 */
  results: IpGeoInfo[]
}

/** SSL 证书信息 */
export interface SslCertInfo {
  domain: string
  issuer: string
  subject: string
  validFrom: string
  validTo: string
  daysRemaining: number
  isExpired: boolean
  isValid: boolean
  san: string[]
  serialNumber: string
  signatureAlgorithm: string
  certificateChain: CertChainItem[]
}

/** 证书链项 */
export interface CertChainItem {
  subject: string
  issuer: string
  isCa: boolean
}

/** SSL 检查结果（包含连接状态） */
export interface SslCheckResult {
  /** 查询的域名 */
  domain: string
  /** 检查的端口 */
  port: number
  /** 连接状态: "https" | "http" | "failed" */
  connectionStatus: "https" | "http" | "failed"
  /** 证书信息（仅当 HTTPS 连接成功时存在） */
  certInfo?: SslCertInfo
  /** 错误信息（连接失败时） */
  error?: string
}

/** HTTP 请求方法 */
export type HttpMethod = "GET" | "HEAD" | "POST" | "PUT" | "DELETE" | "PATCH" | "OPTIONS"

/** HTTP 请求头 */
export interface HttpHeader {
  name: string
  value: string
}

/** HTTP 头检查请求 */
export interface HttpHeaderCheckRequest {
  url: string
  method: HttpMethod
  customHeaders: HttpHeader[]
  body?: string
  contentType?: string
}

/** 安全头分析结果 */
export interface SecurityHeaderAnalysis {
  name: string
  present: boolean
  value?: string
  status: "good" | "warning" | "missing"
  recommendation?: string
}

/** HTTP 头检查结果 */
export interface HttpHeaderCheckResult {
  url: string
  statusCode: number
  statusText: string
  responseTimeMs: number
  headers: HttpHeader[]
  securityAnalysis: SecurityHeaderAnalysis[]
  contentLength?: number
  rawRequest: string
  rawResponse: string
}

/** 查询历史项 */
export interface QueryHistoryItem {
  id: string
  type: "whois" | "dns" | "ip" | "ssl" | "http" | "dns-propagation" | "dnssec"
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

/** DNS 传播检查服务器信息 */
export interface DnsPropagationServer {
  name: string
  ip: string
  region: string
  countryCode: string
}

/** 单个 DNS 服务器的查询结果 */
export interface DnsPropagationServerResult {
  server: DnsPropagationServer
  status: "success" | "timeout" | "error"
  records: DnsLookupRecord[]
  error?: string
  responseTimeMs: number
}

/** DNS 传播检查结果 */
export interface DnsPropagationResult {
  domain: string
  recordType: string
  results: DnsPropagationServerResult[]
  totalTimeMs: number
  consistencyPercentage: number
  uniqueValues: string[]
}

/** DNSSEC DNSKEY 记录 */
export interface DnskeyRecord {
  flags: number
  protocol: number
  algorithm: number
  algorithmName: string
  publicKey: string
  keyTag: number
  keyType: string
}

/** DNSSEC DS 记录 */
export interface DsRecord {
  keyTag: number
  algorithm: number
  algorithmName: string
  digestType: number
  digestTypeName: string
  digest: string
}

/** DNSSEC RRSIG 记录 */
export interface RrsigRecord {
  typeCovered: string
  algorithm: number
  algorithmName: string
  labels: number
  originalTtl: number
  signatureExpiration: string
  signatureInception: string
  keyTag: number
  signerName: string
  signature: string
}

/** DNSSEC 验证结果 */
export interface DnssecResult {
  domain: string
  dnssecEnabled: boolean
  dnskeyRecords: DnskeyRecord[]
  dsRecords: DsRecord[]
  rrsigRecords: RrsigRecord[]
  validationStatus: "secure" | "insecure" | "bogus" | "indeterminate"
  nameserver: string
  responseTimeMs: number
  error?: string
}
