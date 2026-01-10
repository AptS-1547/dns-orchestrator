import { RefreshCw, Search, Star } from "lucide-react"
import { useCallback, useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { useShallow } from "zustand/react/shallow"
import { ProviderIcon } from "@/components/account/ProviderIcon"
import { DomainFavoriteButton } from "@/components/domain/DomainFavoriteButton"
import { DomainTagList } from "@/components/domain/DomainTagList"
import { SelectedTagsList, TagFilterButton } from "@/components/domain/TagFilter"
import { Button } from "@/components/ui/button"
import { EmptyState } from "@/components/ui/empty-state"
import { Input } from "@/components/ui/input"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { type FavoriteDomain, useAccountStore, useDomainStore, useSettingsStore } from "@/stores"

export function FavoriteDomainsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()

  const [searchQuery, setSearchQuery] = useState("")

  // 会话内的收藏域名ID集合（包括已取消但还没离开页面的）
  const [sessionFavoriteIds, setSessionFavoriteIds] = useState<Set<string>>(new Set())

  // 使用 useShallow 优化 accountStore 订阅
  const { accounts } = useAccountStore(
    useShallow((state) => ({
      accounts: state.accounts,
    }))
  )

  // 订阅 domainsByAccount 变化
  const { domainsByAccount, isBackgroundRefreshing, selectedTags } = useDomainStore(
    useShallow((state) => ({
      domainsByAccount: state.domainsByAccount,
      isBackgroundRefreshing: state.isBackgroundRefreshing,
      selectedTags: state.selectedTags,
    }))
  )

  // 主题检测
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)

  // 获取 store 方法
  const refreshAllAccounts = useDomainStore((state) => state.refreshAllAccounts)

  // 初始化和更新会话收藏列表
  useEffect(() => {
    const currentFavorites = new Set<string>()

    Object.entries(domainsByAccount).forEach(([_, cache]) => {
      cache?.domains?.forEach((domain) => {
        if (domain.metadata?.isFavorite) {
          currentFavorites.add(domain.id)
        }
      })
    })

    // 合并：当前收藏 + 会话内已有的
    setSessionFavoriteIds((prev) => new Set([...prev, ...currentFavorites]))
  }, [domainsByAccount])

  // 组件卸载时清空会话列表
  useEffect(() => {
    return () => setSessionFavoriteIds(new Set())
  }, [])

  // 从会话列表构建显示用的收藏域名
  const favorites = useMemo(() => {
    const result: (FavoriteDomain & {
      currentlyFavorited: boolean
      tags: string[]
      color: string | undefined
    })[] = []

    Object.entries(domainsByAccount).forEach(([accountId, cache]) => {
      const account = accounts.find((a) => a.id === accountId)
      if (!(account && cache?.domains)) return

      cache.domains.forEach((domain) => {
        // 只显示会话内曾经收藏的域名
        if (sessionFavoriteIds.has(domain.id)) {
          result.push({
            accountId,
            domainId: domain.id,
            domainName: domain.name,
            accountName: account.name,
            provider: domain.provider,
            // 优先使用 favoritedAt，回退到 updatedAt（兼容旧数据）
            favoritedAt: new Date(
              domain.metadata?.favoritedAt ?? domain.metadata?.updatedAt ?? Date.now()
            ).getTime(),
            currentlyFavorited: domain.metadata?.isFavorite ?? false,
            tags: domain.metadata?.tags ?? [],
            color: domain.metadata?.color,
          })
        }
      })
    })

    return result.sort((a, b) => b.favoritedAt - a.favoritedAt)
  }, [sessionFavoriteIds, domainsByAccount, accounts])

  // 搜索和标签过滤
  const filteredFavorites = useMemo(() => {
    let filtered = favorites

    // 搜索过滤
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter(
        (fav) =>
          fav.domainName.toLowerCase().includes(query) ||
          fav.accountName.toLowerCase().includes(query)
      )
    }

    // 标签筛选（任意匹配）
    if (selectedTags.size > 0) {
      filtered = filtered.filter((fav) =>
        Array.from(selectedTags).some((tag) => fav.tags.includes(tag))
      )
    }

    return filtered
  }, [favorites, searchQuery, selectedTags])

  // 跳转到域名详情
  const handleDomainClick = useCallback(
    (accountId: string, domainId: string) => {
      navigate(`/domains/${accountId}/${domainId}`)
    },
    [navigate]
  )

  // 刷新所有账户
  const handleRefresh = useCallback(() => {
    const validAccounts = accounts.filter((a) => a.status !== "error")
    refreshAllAccounts(validAccounts)
  }, [refreshAllAccounts, accounts])

  return (
    <PageLayout>
      <PageHeader
        title={t("nav.favorites")}
        icon={<Star className="h-5 w-5" />}
        actions={
          <Button
            variant="ghost"
            size="icon"
            onClick={handleRefresh}
            disabled={isBackgroundRefreshing}
            title={t("domains.refresh")}
          >
            <RefreshCw className={cn("h-4 w-4", isBackgroundRefreshing && "animate-spin")} />
          </Button>
        }
      />

      {/* 搜索栏 */}
      <div className="border-b px-4 py-3 sm:px-6">
        <div className="space-y-3">
          <div className="flex flex-col gap-2 sm:flex-row">
            <div className="relative flex-1">
              <Search className="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder={t("favorites.searchPlaceholder")}
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

      {/* 收藏域名列表 */}
      <ScrollArea className="min-h-0 flex-1">
        <div className="scroll-pb-safe p-4 sm:p-6">
          {favorites.length === 0 ? (
            <EmptyState
              icon={<Star className="h-16 w-16" />}
              title={t("favorites.empty")}
              description={t("favorites.emptyDesc")}
              size="large"
            />
          ) : filteredFavorites.length === 0 ? (
            <div className="py-12 text-center text-muted-foreground">{t("common.noMatch")}</div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {filteredFavorites.map((fav) => {
                const hasColor = fav.color && fav.color !== "none"
                const hasTags = fav.tags.length > 0

                return (
                  <button
                    key={fav.domainId}
                    type="button"
                    onClick={() => handleDomainClick(fav.accountId, fav.domainId)}
                    className="relative flex flex-col gap-2 overflow-hidden rounded-lg border p-3 text-left transition-colors hover:bg-accent"
                  >
                    {/* 顶部颜色条 */}
                    {hasColor && (
                      <div
                        className="absolute inset-x-0 top-0 h-1"
                        style={{
                          backgroundColor: getDomainColor(fav.color as DomainColorKey, isDark),
                        }}
                      />
                    )}

                    <div className={cn("flex items-center justify-between", hasColor && "pt-1")}>
                      <ProviderIcon provider={fav.provider} className="h-6 w-6" />
                      <DomainFavoriteButton
                        accountId={fav.accountId}
                        domainId={fav.domainId}
                        isFavorite={fav.currentlyFavorited}
                      />
                    </div>
                    <div className="min-w-0">
                      <div className="truncate font-medium">{fav.domainName}</div>
                      <div className="truncate text-muted-foreground text-xs">
                        {fav.accountName}
                      </div>
                    </div>

                    {/* 标签列表 */}
                    {hasTags && <DomainTagList tags={fav.tags} maxDisplay={2} />}
                  </button>
                )
              })}
            </div>
          )}
        </div>
      </ScrollArea>
    </PageLayout>
  )
}
