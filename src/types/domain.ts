/** 域名状态 */
export type DomainStatus = "active" | "paused" | "pending" | "error"

/** 域名信息 */
export interface Domain {
  id: string
  name: string
  accountId: string
  provider: string
  status: DomainStatus
  recordCount?: number
  createdAt?: string
}
