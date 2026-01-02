import { ChevronRight, Clock, Globe, Loader2, Settings, Star, Users, Wrench } from "lucide-react"
import { useEffect, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { ProviderIcon } from "@/components/account/ProviderIcon"
import { PageContainer } from "@/components/ui/page-container"
import { SectionCard } from "@/components/ui/section-card"
import { getRecentDomains, type RecentDomain } from "@/lib/recent-domains"
import { type FavoriteDomain, useAccountStore, useDomainStore } from "@/stores"

export function HomePage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { accounts, isRestoring } = useAccountStore()
  const { domainsByAccount, getFavoriteDomains } = useDomainStore()
  const [recentDomains, setRecentDomains] = useState<RecentDomain[]>(getRecentDomains)
  const [favoriteDomains, setFavoriteDomains] = useState<FavoriteDomain[]>([])

  // 计算总域名数
  const totalDomains = useMemo(
    () =>
      Object.values(domainsByAccount).reduce(
        (sum, cache) => sum + (cache?.domains?.length ?? 0),
        0
      ),
    [domainsByAccount]
  )

  // 账户变化后重新读取最近域名（清理无效记录后刷新显示）
  // biome-ignore lint/correctness/useExhaustiveDependencies: accounts 用于触发重新读取
  useEffect(() => {
    setRecentDomains(getRecentDomains())
  }, [accounts])

  // 账户或域名缓存变化后更新收藏域名
  // biome-ignore lint/correctness/useExhaustiveDependencies: domainsByAccount 用于触发更新
  useEffect(() => {
    setFavoriteDomains(getFavoriteDomains())
  }, [domainsByAccount, accounts, getFavoriteDomains])

  const handleQuickAccess = (accountId: string, domainId: string) => {
    navigate(`/domains/${accountId}/${domainId}`)
  }

  const quickActions = [
    {
      icon: Globe,
      label: t("nav.domains"),
      description: t("home.manageDomains"),
      onClick: () => navigate("/domains"),
    },
    {
      icon: Wrench,
      label: t("toolbox.title"),
      description: t("home.useTools"),
      onClick: () => navigate("/toolbox"),
    },
    {
      icon: Users,
      label: t("accounts.manage"),
      description: t("home.manageAccounts"),
      onClick: () => navigate("/accounts"),
    },
    {
      icon: Settings,
      label: t("settings.title"),
      description: t("home.configureSettings"),
      onClick: () => navigate("/settings"),
    },
  ]

  return (
    <PageContainer>
      {/* 欢迎 */}
      <div className="mb-6">
        <h1 className="font-semibold text-2xl">{t("home.welcome")}</h1>
        <p className="mt-1 text-muted-foreground">{t("home.welcomeDesc")}</p>
        <div className="mt-2 flex items-center gap-1 text-muted-foreground text-sm">
          {isRestoring ? (
            <>
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              <span>{t("home.restoringAccounts")}</span>
            </>
          ) : (
            <>
              <Users className="h-3.5 w-3.5" />
              <span>
                {accounts.length} {t("home.totalAccounts")}
              </span>
              <span className="mx-1">·</span>
              <Globe className="h-3.5 w-3.5" />
              <span>
                {totalDomains} {t("home.totalDomains")}
              </span>
            </>
          )}
        </div>
      </div>

      {/* 最近访问 */}
      {recentDomains.length > 0 && (
        <SectionCard
          title={t("home.recentDomains")}
          icon={<Clock className="h-5 w-5" />}
          description={t("home.recentDomainsDesc")}
          className="mb-4"
        >
          {/* 移动端：紧凑列表 */}
          <div className="flex flex-col divide-y sm:hidden">
            {recentDomains.map((domain) => (
              <button
                key={domain.domainId}
                type="button"
                onClick={() => handleQuickAccess(domain.accountId, domain.domainId)}
                className="flex items-center gap-3 px-2 py-3 text-left transition-colors hover:bg-accent"
              >
                <ProviderIcon provider={domain.provider} className="h-5 w-5 shrink-0" />
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium text-sm">{domain.domainName}</div>
                  <div className="truncate text-muted-foreground text-xs">{domain.accountName}</div>
                </div>
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              </button>
            ))}
          </div>
          {/* 桌面端：网格卡片 */}
          <div className="hidden gap-3 sm:grid sm:grid-cols-2 lg:grid-cols-3">
            {recentDomains.map((domain) => (
              <button
                key={domain.domainId}
                type="button"
                onClick={() => handleQuickAccess(domain.accountId, domain.domainId)}
                className="flex flex-col items-center gap-2 rounded-lg border p-3 text-center transition-colors hover:bg-accent"
              >
                <ProviderIcon provider={domain.provider} className="h-6 w-6" />
                <div className="w-full min-w-0">
                  <div className="truncate font-medium">{domain.domainName}</div>
                  <div className="truncate text-muted-foreground text-xs">{domain.accountName}</div>
                </div>
              </button>
            ))}
          </div>
        </SectionCard>
      )}

      {/* 收藏域名 */}
      {favoriteDomains.length > 0 && (
        <SectionCard
          title={t("home.favoriteDomains")}
          icon={<Star className="h-5 w-5" />}
          description={t("home.favoriteDomainsDesc")}
          className="mb-4"
        >
          {/* 移动端：紧凑列表 */}
          <div className="flex flex-col divide-y sm:hidden">
            {favoriteDomains.map((domain) => (
              <button
                key={domain.domainId}
                type="button"
                onClick={() => handleQuickAccess(domain.accountId, domain.domainId)}
                className="flex items-center gap-3 px-2 py-3 text-left transition-colors hover:bg-accent"
              >
                <ProviderIcon provider={domain.provider} className="h-5 w-5 shrink-0" />
                <div className="min-w-0 flex-1">
                  <div className="truncate font-medium text-sm">{domain.domainName}</div>
                  <div className="truncate text-muted-foreground text-xs">{domain.accountName}</div>
                </div>
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              </button>
            ))}
          </div>
          {/* 桌面端：网格卡片 */}
          <div className="hidden gap-3 sm:grid sm:grid-cols-2 lg:grid-cols-3">
            {favoriteDomains.map((domain) => (
              <button
                key={domain.domainId}
                type="button"
                onClick={() => handleQuickAccess(domain.accountId, domain.domainId)}
                className="flex flex-col items-center gap-2 rounded-lg border p-3 text-center transition-colors hover:bg-accent"
              >
                <ProviderIcon provider={domain.provider} className="h-6 w-6" />
                <div className="w-full min-w-0">
                  <div className="truncate font-medium">{domain.domainName}</div>
                  <div className="truncate text-muted-foreground text-xs">{domain.accountName}</div>
                </div>
              </button>
            ))}
          </div>
        </SectionCard>
      )}

      {/* 快捷操作 */}
      <SectionCard title={t("home.quickActions")} description={t("home.quickActionsDesc")}>
        {/* 移动端：紧凑列表 */}
        <div className="flex flex-col divide-y sm:hidden">
          {quickActions.map((action) => (
            <button
              key={action.label}
              type="button"
              onClick={action.onClick}
              className="flex items-center gap-3 px-2 py-3 text-left transition-colors hover:bg-accent"
            >
              <action.icon className="h-5 w-5 shrink-0" />
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-sm">{action.label}</div>
                <div className="truncate text-muted-foreground text-xs">{action.description}</div>
              </div>
              <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
            </button>
          ))}
        </div>
        {/* 桌面端：网格卡片 */}
        <div className="hidden gap-3 sm:grid sm:grid-cols-2">
          {quickActions.map((action) => (
            <button
              key={action.label}
              type="button"
              onClick={action.onClick}
              className="flex h-auto items-center justify-start gap-3 rounded-lg border p-3 text-left transition-colors hover:bg-accent"
            >
              <action.icon className="h-5 w-5 shrink-0" />
              <div>
                <div className="font-medium">{action.label}</div>
                <div className="text-muted-foreground text-xs">{action.description}</div>
              </div>
            </button>
          ))}
        </div>
      </SectionCard>
    </PageContainer>
  )
}
