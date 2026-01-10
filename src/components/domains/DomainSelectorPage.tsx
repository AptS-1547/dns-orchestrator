import { Globe, ListChecks, RefreshCw, Search } from "lucide-react"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useShallow } from "zustand/react/shallow"
import { DomainBatchActionBar } from "@/components/domain/DomainBatchActionBar"
import { SelectedTagsList, TagFilterButton } from "@/components/domain/TagFilter"
import { Button } from "@/components/ui/button"
import { EmptyState } from "@/components/ui/empty-state"
import { Input } from "@/components/ui/input"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { useAccountStore, useDomainStore } from "@/stores"
import { DomainAccountGroup } from "./DomainAccountGroup"

export function DomainSelectorPage() {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState("")
  const scrollAreaRef = useRef<HTMLDivElement>(null)

  // Account store
  const { accounts, isLoading: isAccountsLoading } = useAccountStore(
    useShallow((state) => ({
      accounts: state.accounts,
      isLoading: state.isLoading,
    }))
  )

  // Domain store - state
  const {
    isBackgroundRefreshing,
    scrollPosition,
    isBatchMode,
    selectedDomainKeys,
    isBatchOperating,
    domainsByAccount,
  } = useDomainStore(
    useShallow((state) => ({
      isBackgroundRefreshing: state.isBackgroundRefreshing,
      scrollPosition: state.scrollPosition,
      isBatchMode: state.isBatchMode,
      selectedDomainKeys: state.selectedDomainKeys,
      isBatchOperating: state.isBatchOperating,
      domainsByAccount: state.domainsByAccount,
    }))
  )

  // Domain store - actions
  const refreshAllAccounts = useDomainStore((state) => state.refreshAllAccounts)
  const setScrollPosition = useDomainStore((state) => state.setScrollPosition)
  const toggleBatchMode = useDomainStore((state) => state.toggleBatchMode)
  const clearDomainSelection = useDomainStore((state) => state.clearDomainSelection)
  const batchAddTags = useDomainStore((state) => state.batchAddTags)
  const batchRemoveTags = useDomainStore((state) => state.batchRemoveTags)
  const batchSetTags = useDomainStore((state) => state.batchSetTags)
  const getAllUsedTags = useDomainStore((state) => state.getAllUsedTags)
  const getDomainsForAccount = useDomainStore((state) => state.getDomainsForAccount)

  // Valid accounts (exclude error status)
  const validAccounts = useMemo(() => accounts.filter((a) => a.status !== "error"), [accounts])

  // Scroll position restoration
  useEffect(() => {
    if (scrollAreaRef.current && scrollPosition > 0) {
      const viewport = scrollAreaRef.current.querySelector("[data-radix-scroll-area-viewport]")
      if (viewport) viewport.scrollTop = scrollPosition
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Scroll position saving
  useEffect(() => {
    const viewport = scrollAreaRef.current?.querySelector("[data-radix-scroll-area-viewport]")
    if (!viewport) return

    const handleScroll = () => setScrollPosition(viewport.scrollTop)
    viewport.addEventListener("scroll", handleScroll)
    return () => viewport.removeEventListener("scroll", handleScroll)
  }, [setScrollPosition])

  const handleRefreshAll = useCallback(() => {
    refreshAllAccounts(validAccounts)
  }, [refreshAllAccounts, validAccounts])

  // Batch action helpers
  const selectedDomainsTags = useMemo(() => {
    const tagsSet = new Set<string>()
    selectedDomainKeys.forEach((key) => {
      const [accountId, domainId] = key.split("::")
      const domains = getDomainsForAccount(accountId)
      const domain = domains.find((d) => d.id === domainId)
      domain?.metadata?.tags?.forEach((tag) => tagsSet.add(tag))
    })
    return Array.from(tagsSet).sort()
  }, [selectedDomainKeys, getDomainsForAccount])

  const allTags = useMemo(() => getAllUsedTags(), [domainsByAccount, getAllUsedTags])

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
              title={isBatchMode ? t("domain.batch.exitMode") : t("domain.batch.enterMode")}
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

      {/* Search toolbar */}
      <div className="border-b px-4 py-3 sm:px-6">
        <div className="space-y-3">
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
          <SelectedTagsList />
        </div>
      </div>

      {/* Account domain list */}
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
            accounts.map((account) => (
              <DomainAccountGroup
                key={account.id}
                account={account}
                searchQuery={searchQuery}
                isBatchMode={isBatchMode}
              />
            ))
          )}
        </div>
      </ScrollArea>

      {/* Batch action bar */}
      <DomainBatchActionBar
        selectedCount={selectedDomainKeys.size}
        isOperating={isBatchOperating}
        onClearSelection={clearDomainSelection}
        onAddTags={batchAddTags}
        onRemoveTags={batchRemoveTags}
        onSetTags={batchSetTags}
        selectedDomainsTags={selectedDomainsTags}
        allTags={allTags}
      />
    </PageLayout>
  )
}
