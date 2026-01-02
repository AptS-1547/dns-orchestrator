import {
  ChevronRight,
  Globe,
  ListChecks,
  Loader2,
  Plus,
  RefreshCw,
  Search,
  TriangleAlert,
} from "lucide-react"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { useShallow } from "zustand/react/shallow"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
import { DomainBatchActionBar } from "@/components/domain/DomainBatchActionBar"
import { DomainFavoriteButton } from "@/components/domain/DomainFavoriteButton"
import { DomainMetadataEditor } from "@/components/domain/DomainMetadataEditor"
import { DomainTagList } from "@/components/domain/DomainTagList"
import { SelectedTagsList, TagFilterButton } from "@/components/domain/TagFilter"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { EmptyState } from "@/components/ui/empty-state"
import { Input } from "@/components/ui/input"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { useAccountStore, useDomainStore, useSettingsStore } from "@/stores"
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

  // 获取主题状态（用于颜色标记显示）
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)

  // 使用 useShallow 优化 accountStore 订阅
  const { accounts, isLoading: isAccountsLoading } = useAccountStore(
    useShallow((state) => ({
      accounts: state.accounts,
      isLoading: state.isLoading,
    }))
  )

  // 使用 useShallow 优化 domainStore 订阅
  const {
    domainsByAccount,
    isBackgroundRefreshing,
    expandedAccounts,
    scrollPosition,
    selectedTags,
    isBatchMode,
    selectedDomainKeys,
    isBatchOperating,
  } = useDomainStore(
    useShallow((state) => ({
      domainsByAccount: state.domainsByAccount,
      isBackgroundRefreshing: state.isBackgroundRefreshing,
      expandedAccounts: state.expandedAccounts,
      scrollPosition: state.scrollPosition,
      selectedTags: state.selectedTags,
      isBatchMode: state.isBatchMode,
      selectedDomainKeys: state.selectedDomainKeys,
      isBatchOperating: state.isBatchOperating,
    }))
  )

  // actions 单独获取
  const refreshAccount = useDomainStore((state) => state.refreshAccount)
  const refreshAllAccounts = useDomainStore((state) => state.refreshAllAccounts)
  const loadMoreDomains = useDomainStore((state) => state.loadMoreDomains)
  const getDomainsForAccount = useDomainStore((state) => state.getDomainsForAccount)
  const isAccountLoading = useDomainStore((state) => state.isAccountLoading)
  const isAccountLoadingMore = useDomainStore((state) => state.isAccountLoadingMore)
  const hasMoreDomains = useDomainStore((state) => state.hasMoreDomains)
  const toggleExpandedAccount = useDomainStore((state) => state.toggleExpandedAccount)
  const setScrollPosition = useDomainStore((state) => state.setScrollPosition)
  const toggleBatchMode = useDomainStore((state) => state.toggleBatchMode)
  const toggleDomainSelection = useDomainStore((state) => state.toggleDomainSelection)
  const clearDomainSelection = useDomainStore((state) => state.clearDomainSelection)
  const batchAddTags = useDomainStore((state) => state.batchAddTags)
  const batchRemoveTags = useDomainStore((state) => state.batchRemoveTags)
  const batchSetTags = useDomainStore((state) => state.batchSetTags)

  const [searchQuery, setSearchQuery] = useState("")
  const scrollAreaRef = useRef<HTMLDivElement>(null)

  // 有效账户（排除错误状态）
  const validAccounts = useMemo(
    () => accounts.filter((account) => account.status !== "error"),
    [accounts]
  )

  // 恢复滚动位置（组件挂载时）
  useEffect(() => {
    if (scrollAreaRef.current && scrollPosition > 0) {
      const viewport = scrollAreaRef.current.querySelector("[data-radix-scroll-area-viewport]")
      if (viewport) {
        viewport.scrollTop = scrollPosition
      }
    }
    // 只在挂载时执行一次，恢复上次的滚动位置
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // 监听滚动事件，保存滚动位置
  useEffect(() => {
    const viewport = scrollAreaRef.current?.querySelector("[data-radix-scroll-area-viewport]")
    if (!viewport) return

    const handleScroll = () => {
      setScrollPosition(viewport.scrollTop)
    }

    viewport.addEventListener("scroll", handleScroll)
    return () => viewport.removeEventListener("scroll", handleScroll)
  }, [setScrollPosition])

  // 切换账户展开状态
  const toggleAccount = useCallback(
    (accountId: string) => {
      toggleExpandedAccount(accountId)
      // 如果没有缓存，加载域名
      if (!(expandedAccounts.has(accountId) || domainsByAccount[accountId])) {
        refreshAccount(accountId).catch(() => {})
      }
    },
    [domainsByAccount, refreshAccount, expandedAccounts, toggleExpandedAccount]
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
      let filtered = domains

      // 搜索查询过滤
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase()
        filtered = filtered.filter((domain) => domain.name.toLowerCase().includes(query))
      }

      // 标签筛选（任意匹配）
      if (selectedTags.size > 0) {
        filtered = filtered.filter((domain) => {
          const domainTags = domain.metadata?.tags ?? []
          return Array.from(selectedTags).some((tag) => domainTags.includes(tag))
        })
      }

      return filtered
    },
    [searchQuery, selectedTags]
  )

  // 获取所有选中域名的标签并集
  const selectedDomainsTags = useMemo(() => {
    const tagsSet = new Set<string>()
    selectedDomainKeys.forEach((key) => {
      const [accountId, domainId] = key.split("::")
      const domains = getDomainsForAccount(accountId)
      const domain = domains.find((d) => d.id === domainId)
      if (domain?.metadata?.tags) {
        for (const tag of domain.metadata.tags) {
          tagsSet.add(tag)
        }
      }
    })
    return Array.from(tagsSet).sort()
  }, [selectedDomainKeys, getDomainsForAccount])

  // 渲染域名项
  const renderDomainItem = (domain: Domain, accountId: string) => {
    const config = statusConfig[domain.status] ?? statusConfig.active
    const domainKey = `${accountId}::${domain.id}`
    const isSelected = isBatchMode && selectedDomainKeys.has(domainKey)

    return (
      <button
        key={domain.id}
        type="button"
        onClick={() => {
          if (isBatchMode) {
            toggleDomainSelection(accountId, domain.id)
          } else {
            handleSelectDomain(accountId, domain.id)
          }
        }}
        className={cn(
          "flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors",
          "hover:bg-accent hover:text-accent-foreground",
          isBatchMode && isSelected && "bg-accent ring-2 ring-primary"
        )}
      >
        <div className="flex w-full items-center gap-2">
          {isBatchMode && <Checkbox checked={isSelected} className="pointer-events-none" />}
          {!isBatchMode && (
            <DomainFavoriteButton
              accountId={accountId}
              domainId={domain.id}
              isFavorite={domain.metadata?.isFavorite ?? false}
            />
          )}
          {/* 颜色标记（固定占位以保持对齐） */}
          <div
            className="h-3 w-0.5 shrink-0 rounded-full"
            style={{
              backgroundColor: domain.metadata?.color
                ? getDomainColor(domain.metadata.color as DomainColorKey, isDark)
                : "transparent",
            }}
            aria-label={domain.metadata?.color ? `Color: ${domain.metadata.color}` : undefined}
          />
          <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />
          <div className="flex min-w-0 flex-1 items-center">
            <div className="flex min-w-0 items-center gap-2">
              <span className="min-w-[120px] shrink-0 truncate">{domain.name}</span>
              <DomainTagList
                className="shrink"
                tags={domain.metadata?.tags ?? []}
                onClickTag={(tag) => {
                  const { setSelectedTags, selectedTags } = useDomainStore.getState()
                  const newTags = new Set(selectedTags)
                  newTags.add(tag)
                  setSelectedTags(Array.from(newTags))
                }}
              />
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-1">
            <Badge variant={config.variant}>{t(config.labelKey)}</Badge>
            {!isBatchMode && (
              <DomainMetadataEditor
                accountId={accountId}
                domainId={domain.id}
                currentMetadata={domain.metadata}
              >
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <Plus className="h-4 w-4" />
                </Button>
              </DomainMetadataEditor>
            )}
          </div>
        </div>
      </button>
    )
  }

  // 渲染账户内容区域
  const renderAccountContent = (accountId: string) => {
    const domains = getDomainsForAccount(accountId)
    const filteredDomains = getFilteredDomains(domains)
    const isLoading = isAccountLoading(accountId)
    const isLoadingMore = isAccountLoadingMore(accountId)
    const hasMore = hasMoreDomains(accountId)
    const hasCachedData = domains.length > 0

    if (isLoading && !hasCachedData) {
      return (
        <div className="space-y-2 py-2">
          <Skeleton className="h-9 w-full" />
          <Skeleton className="h-9 w-full" />
          <Skeleton className="h-9 w-3/4" />
        </div>
      )
    }

    if (filteredDomains.length === 0) {
      return (
        <div className="py-4 text-center text-muted-foreground text-sm">
          {searchQuery ? t("common.noMatch") : t("domain.noDomains")}
        </div>
      )
    }

    return (
      <div className="space-y-1">
        {filteredDomains.map((domain) => renderDomainItem(domain, accountId))}
        {hasMore && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              handleLoadMore(accountId)
            }}
            disabled={isLoadingMore}
            className="flex w-full items-center justify-center gap-2 py-2 text-muted-foreground text-sm transition-colors hover:text-foreground"
          >
            {isLoadingMore ? <Loader2 className="h-4 w-4 animate-spin" /> : t("common.loadMore")}
          </button>
        )}
      </div>
    )
  }

  // 渲染账户组
  const renderAccountGroup = (account: Account) => {
    const isExpanded = expandedAccounts.has(account.id)
    const hasError = account.status === "error"

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
            <div className="border-t px-3 py-2">{renderAccountContent(account.id)}</div>
          </CollapsibleContent>
        </div>
      </Collapsible>
    )
  }

  return (
    <PageLayout>
      <PageHeader
        title={t("nav.domains")}
        icon={<Globe className="h-5 w-5" />}
        actions={
          <div className="flex items-center gap-2">
            <Button
              variant={isBatchMode ? "default" : "outline"}
              size="icon"
              onClick={toggleBatchMode}
              title={isBatchMode ? "退出批量模式" : "批量选择"}
            >
              <ListChecks className="h-4 w-4" />
            </Button>
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
        }
      />

      {/* 搜索栏 */}
      <div className="border-b px-4 py-3 sm:px-6">
        <div className="space-y-3">
          {/* 第一行：搜索框 + 筛选按钮 */}
          <div className="flex flex-col gap-2 sm:flex-row">
            <div className="relative flex-1">
              <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder={t("domains.searchPlaceholder")}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-9"
              />
            </div>
            <TagFilterButton />
          </div>

          {/* 第二行：已选标签（仅在有标签时显示）*/}
          <SelectedTagsList />
        </div>
      </div>

      {/* 账户域名列表 */}
      <ScrollArea className="min-h-0 flex-1" ref={scrollAreaRef}>
        <div className="scroll-pb-safe space-y-3 p-4 sm:p-6">
          {isAccountsLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-16 w-full rounded-lg" />
              <Skeleton className="h-16 w-full rounded-lg" />
            </div>
          ) : accounts.length === 0 ? (
            <EmptyState
              icon={<Globe className="h-16 w-16" />}
              title={t("accounts.empty")}
              description={t("accounts.emptyDesc")}
              size="large"
            />
          ) : (
            accounts.map((account) => renderAccountGroup(account))
          )}
        </div>
      </ScrollArea>

      {/* 批量操作工具栏 */}
      <DomainBatchActionBar
        selectedCount={selectedDomainKeys.size}
        isOperating={isBatchOperating}
        onClearSelection={clearDomainSelection}
        onAddTags={batchAddTags}
        onRemoveTags={batchRemoveTags}
        onSetTags={batchSetTags}
        selectedDomainsTags={selectedDomainsTags}
      />
    </PageLayout>
  )
}
