import { ChevronRight, Loader2, TriangleAlert } from "lucide-react"
import { useCallback } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { useDomainStore, useSettingsStore } from "@/stores"
import type { Account } from "@/types"
import { DomainItem } from "./DomainItem"
import { useFilteredDomains } from "./hooks/useFilteredDomains"

interface DomainAccountGroupProps {
  account: Account
  searchQuery: string
  isBatchMode: boolean
}

export function DomainAccountGroup({ account, searchQuery, isBatchMode }: DomainAccountGroupProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()

  // Theme for color display
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)

  // Domain store state
  const expandedAccounts = useDomainStore((state) => state.expandedAccounts)
  const domainsByAccount = useDomainStore((state) => state.domainsByAccount)
  const selectedDomainKeys = useDomainStore((state) => state.selectedDomainKeys)

  // Domain store actions
  const refreshAccount = useDomainStore((state) => state.refreshAccount)
  const loadMoreDomains = useDomainStore((state) => state.loadMoreDomains)
  const isAccountLoading = useDomainStore((state) => state.isAccountLoading)
  const isAccountLoadingMore = useDomainStore((state) => state.isAccountLoadingMore)
  const hasMoreDomains = useDomainStore((state) => state.hasMoreDomains)
  const toggleExpandedAccount = useDomainStore((state) => state.toggleExpandedAccount)
  const toggleDomainSelection = useDomainStore((state) => state.toggleDomainSelection)
  const setSelectedTags = useDomainStore((state) => state.setSelectedTags)

  // Filtered domains
  const filteredDomains = useFilteredDomains({ accountId: account.id, searchQuery })

  const isExpanded = expandedAccounts.has(account.id)
  const hasError = account.status === "error"
  const isLoading = isAccountLoading(account.id)
  const isLoadingMore = isAccountLoadingMore(account.id)
  const hasMore = hasMoreDomains(account.id)
  const hasCachedData = (domainsByAccount[account.id]?.domains?.length ?? 0) > 0

  // Toggle account expansion
  const toggleAccount = useCallback(() => {
    if (hasError) return
    toggleExpandedAccount(account.id)
    // Load domains if no cache
    if (!(expandedAccounts.has(account.id) || domainsByAccount[account.id])) {
      refreshAccount(account.id).catch(() => {})
    }
  }, [
    account.id,
    hasError,
    expandedAccounts,
    domainsByAccount,
    toggleExpandedAccount,
    refreshAccount,
  ])

  // Select domain (navigate)
  const handleSelectDomain = useCallback(
    (domainId: string) => {
      navigate(`/domains/${account.id}/${domainId}`)
    },
    [account.id, navigate]
  )

  // Toggle domain selection (batch mode)
  const handleToggleSelection = useCallback(
    (domainId: string) => {
      toggleDomainSelection(account.id, domainId)
    },
    [account.id, toggleDomainSelection]
  )

  // Click tag to add to filter
  const handleClickTag = useCallback(
    (tag: string) => {
      const { selectedTags } = useDomainStore.getState()
      const newTags = new Set(selectedTags)
      newTags.add(tag)
      setSelectedTags(Array.from(newTags))
    },
    [setSelectedTags]
  )

  // Load more domains
  const handleLoadMore = useCallback(() => {
    loadMoreDomains(account.id)
  }, [account.id, loadMoreDomains])

  // Render content area
  const renderContent = () => {
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
        {filteredDomains.map((domain) => {
          const domainKey = `${account.id}::${domain.id}`
          const isSelected = isBatchMode && selectedDomainKeys.has(domainKey)

          return (
            <DomainItem
              key={domain.id}
              domain={domain}
              accountId={account.id}
              isBatchMode={isBatchMode}
              isSelected={isSelected}
              isDark={isDark}
              onSelect={() => handleSelectDomain(domain.id)}
              onToggleSelection={() => handleToggleSelection(domain.id)}
              onClickTag={handleClickTag}
            />
          )
        })}
        {hasMore && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation()
              handleLoadMore()
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

  return (
    <Collapsible open={isExpanded} onOpenChange={toggleAccount}>
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
          <div className="border-t px-3 py-2">{renderContent()}</div>
        </CollapsibleContent>
      </div>
    </Collapsible>
  )
}
