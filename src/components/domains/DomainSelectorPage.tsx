import {
  ArrowLeft,
  ChevronRight,
  Globe,
  Loader2,
  RefreshCw,
  Search,
  TriangleAlert,
} from "lucide-react"
import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { useAccountStore, useDomainStore } from "@/stores"
import type { Account, Domain, DomainStatus } from "@/types"

const statusConfig: Record<
  DomainStatus,
  { labelKey: string; variant: "default" | "secondary" | "destructive" | "outline" }
> = {
  active: { labelKey: "domain.status.active", variant: "default" },
  paused: { labelKey: "domain.status.paused", variant: "secondary" },
  pending: { labelKey: "domain.status.pending", variant: "outline" },
  error: { labelKey: "domain.status.error", variant: "destructive" },
  unknown: { labelKey: "domain.status.unknown", variant: "outline" },
}

export function DomainSelectorPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { accounts, isLoading: isAccountsLoading } = useAccountStore()
  const {
    domainsByAccount,
    isBackgroundRefreshing,
    refreshAccount,
    refreshAllAccounts,
    loadMoreDomains,
    getDomainsForAccount,
    isAccountLoading,
    isAccountLoadingMore,
    hasMoreDomains,
  } = useDomainStore()

  const [searchQuery, setSearchQuery] = useState("")
  const [expandedAccounts, setExpandedAccounts] = useState<Set<string>>(new Set())

  // 有效账户（排除错误状态）
  const validAccounts = useMemo(
    () => accounts.filter((account) => account.status !== "error"),
    [accounts]
  )

  // 默认展开第一个账户
  useEffect(() => {
    if (validAccounts.length > 0 && expandedAccounts.size === 0) {
      const firstAccountId = validAccounts[0].id
      setExpandedAccounts(new Set([firstAccountId]))
      // 如果该账户没有缓存，则加载
      if (!domainsByAccount[firstAccountId]) {
        refreshAccount(firstAccountId).catch(() => {})
      }
    }
  }, [validAccounts, expandedAccounts.size, domainsByAccount, refreshAccount])

  // 切换账户展开状态
  const toggleAccount = useCallback(
    (accountId: string) => {
      setExpandedAccounts((prev) => {
        const next = new Set(prev)
        if (next.has(accountId)) {
          next.delete(accountId)
        } else {
          next.add(accountId)
          // 如果没有缓存，加载域名
          if (!domainsByAccount[accountId]) {
            refreshAccount(accountId).catch(() => {})
          }
        }
        return next
      })
    },
    [domainsByAccount, refreshAccount]
  )

  // 手动刷新所有账户
  const handleRefreshAll = useCallback(() => {
    refreshAllAccounts(validAccounts)
  }, [refreshAllAccounts, validAccounts])

  // 选择域名
  const handleSelectDomain = useCallback(
    (accountId: string, domainId: string) => {
      navigate(`/domains/${accountId}/${domainId}`)
    },
    [navigate]
  )

  // 加载更多域名
  const handleLoadMore = useCallback(
    (accountId: string) => {
      loadMoreDomains(accountId)
    },
    [loadMoreDomains]
  )

  // 过滤域名
  const getFilteredDomains = useCallback(
    (domains: Domain[]) => {
      if (!searchQuery.trim()) return domains
      const query = searchQuery.toLowerCase()
      return domains.filter((domain) => domain.name.toLowerCase().includes(query))
    },
    [searchQuery]
  )

  // 渲染域名项
  const renderDomainItem = (domain: Domain, accountId: string) => {
    const config = statusConfig[domain.status] ?? statusConfig.active
    return (
      <button
        key={domain.id}
        type="button"
        onClick={() => handleSelectDomain(accountId, domain.id)}
        className={cn(
          "flex w-full items-center gap-3 rounded-md px-3 py-2 text-left transition-colors",
          "hover:bg-accent hover:text-accent-foreground"
        )}
      >
        <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />
        <span className="flex-1 truncate">{domain.name}</span>
        <Badge variant={config.variant} className="shrink-0">
          {t(config.labelKey)}
        </Badge>
      </button>
    )
  }

  // 渲染账户组
  const renderAccountGroup = (account: Account) => {
    const isExpanded = expandedAccounts.has(account.id)
    const hasError = account.status === "error"
    const domains = getDomainsForAccount(account.id)
    const filteredDomains = getFilteredDomains(domains)
    const isLoading = isAccountLoading(account.id)
    const isLoadingMore = isAccountLoadingMore(account.id)
    const hasMore = hasMoreDomains(account.id)
    const hasCachedData = domains.length > 0

    return (
      <Collapsible
        key={account.id}
        open={isExpanded}
        onOpenChange={() => !hasError && toggleAccount(account.id)}
      >
        <div className="rounded-lg border bg-card">
          <CollapsibleTrigger asChild disabled={hasError}>
            <button
              type="button"
              className={cn(
                "flex w-full items-center gap-3 rounded-lg p-3 text-left transition-colors",
                "hover:bg-accent/50",
                hasError && "cursor-not-allowed opacity-60"
              )}
            >
              <ChevronRight
                className={cn(
                  "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
                  isExpanded && "rotate-90"
                )}
              />
              <div className="flex h-9 w-9 items-center justify-center rounded-md bg-muted">
                <ProviderIcon provider={account.provider} className="h-5 w-5" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="truncate font-medium">{account.name}</span>
                  {hasError && <TriangleAlert className="h-4 w-4 shrink-0 text-destructive" />}
                </div>
                <span className="text-muted-foreground text-sm">
                  {getProviderName(account.provider)}
                </span>
              </div>
            </button>
          </CollapsibleTrigger>

          <CollapsibleContent>
            <div className="border-t px-3 py-2">
              {isLoading && !hasCachedData ? (
                <div className="space-y-2 py-2">
                  <Skeleton className="h-9 w-full" />
                  <Skeleton className="h-9 w-full" />
                  <Skeleton className="h-9 w-3/4" />
                </div>
              ) : filteredDomains.length === 0 ? (
                <div className="py-4 text-center text-muted-foreground text-sm">
                  {searchQuery ? t("common.noMatch") : t("domain.noDomains")}
                </div>
              ) : (
                <div className="space-y-1">
                  {filteredDomains.map((domain) => renderDomainItem(domain, account.id))}
                  {hasMore && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation()
                        handleLoadMore(account.id)
                      }}
                      disabled={isLoadingMore}
                      className="flex w-full items-center justify-center gap-2 py-2 text-muted-foreground text-sm transition-colors hover:text-foreground"
                    >
                      {isLoadingMore ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        t("common.loadMore")
                      )}
                    </button>
                  )}
                </div>
              )}
            </div>
          </CollapsibleContent>
        </div>
      </Collapsible>
    )
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b bg-background px-4 py-3 sm:gap-3 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={() => navigate("/")}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <Globe className="h-5 w-5 text-primary" />
        <h2 className="flex-1 font-semibold text-xl">{t("nav.domains")}</h2>
        <Button
          variant="ghost"
          size="icon"
          onClick={handleRefreshAll}
          disabled={isBackgroundRefreshing}
          title={t("domains.refresh")}
        >
          <RefreshCw className={cn("h-4 w-4", isBackgroundRefreshing && "animate-spin")} />
        </Button>
      </div>

      {/* 搜索栏 */}
      <div className="border-b px-4 py-3 sm:px-6">
        <div className="relative">
          <Search className="-translate-y-1/2 absolute top-1/2 left-3 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder={t("domains.searchPlaceholder")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
      </div>

      {/* 账户域名列表 */}
      <ScrollArea className="min-h-0 flex-1">
        <div className="space-y-3 p-4 sm:p-6">
          {isAccountsLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full rounded-lg" />
              <Skeleton className="h-16 w-full rounded-lg" />
            </div>
          ) : accounts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Globe className="mb-4 h-16 w-16 text-muted-foreground/30" />
              <h3 className="mb-2 font-medium text-lg">{t("accounts.empty")}</h3>
              <p className="max-w-sm text-muted-foreground text-sm">{t("accounts.emptyDesc")}</p>
            </div>
          ) : (
            accounts.map((account) => renderAccountGroup(account))
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
