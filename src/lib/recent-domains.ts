import { LIMITS } from "@/constants"
import { type RecentDomain, storage } from "@/services/storage"

// 重新导出类型，保持向后兼容
export type { RecentDomain } from "@/services/storage"

export function getRecentDomains(): RecentDomain[] {
  try {
    return storage.get("recentDomains") ?? []
  } catch {
    return []
  }
}

export function addRecentDomain(domain: Omit<RecentDomain, "timestamp">) {
  const recent = getRecentDomains()
  const filtered = recent.filter((d) => d.domainId !== domain.domainId)
  const updated = [{ ...domain, timestamp: Date.now() }, ...filtered].slice(
    0,
    LIMITS.MAX_RECENT_DOMAINS
  )
  storage.set("recentDomains", updated)
}

export function removeRecentDomainsByAccount(accountId: string) {
  const recent = getRecentDomains()
  const filtered = recent.filter((d) => d.accountId !== accountId)
  storage.set("recentDomains", filtered)
}

export function cleanupInvalidRecentDomains(validAccountIds: string[]) {
  const recent = getRecentDomains()
  const filtered = recent.filter((d) => validAccountIds.includes(d.accountId))
  if (filtered.length !== recent.length) {
    storage.set("recentDomains", filtered)
  }
}
