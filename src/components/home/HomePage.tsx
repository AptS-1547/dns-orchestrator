import { ChevronRight, Clock, Globe, Settings, Users, Wrench } from "lucide-react"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { ProviderIcon } from "@/components/account/ProviderIcon"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { LIMITS, STORAGE_KEYS } from "@/constants"
import { useAccountStore, useDomainStore } from "@/stores"

interface RecentDomain {
  accountId: string
  domainId: string
  domainName: string
  accountName: string
  provider: string
  timestamp: number
}

export function getRecentDomains(): RecentDomain[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEYS.RECENT_DOMAINS)
    return stored ? JSON.parse(stored) : []
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
  localStorage.setItem(STORAGE_KEYS.RECENT_DOMAINS, JSON.stringify(updated))
}

export function removeRecentDomainsByAccount(accountId: string) {
  const recent = getRecentDomains()
  const filtered = recent.filter((d) => d.accountId !== accountId)
  localStorage.setItem(STORAGE_KEYS.RECENT_DOMAINS, JSON.stringify(filtered))
}

export function cleanupInvalidRecentDomains(validAccountIds: string[]) {
  const recent = getRecentDomains()
  const filtered = recent.filter((d) => validAccountIds.includes(d.accountId))
  if (filtered.length !== recent.length) {
    localStorage.setItem(STORAGE_KEYS.RECENT_DOMAINS, JSON.stringify(filtered))
  }
}

export function HomePage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { accounts } = useAccountStore()
  const { domainsByAccount } = useDomainStore()
  const [recentDomains, setRecentDomains] = useState<RecentDomain[]>([])

  // 计算总域名数
  const totalDomains = Object.values(domainsByAccount).reduce(
    (sum, cache) => sum + (cache?.domains?.length ?? 0),
    0
  )

  // 账户变化后重新读取最近域名（清理无效记录后刷新显示）
  // biome-ignore lint/correctness/useExhaustiveDependencies: accounts 用于触发重新读取
  useEffect(() => {
    setRecentDomains(getRecentDomains())
  }, [accounts])

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
    <ScrollArea className="flex-1">
      <div className="scroll-pb-safe p-4 sm:p-6">
        {/* 欢迎 */}
        <div className="mb-6">
          <h1 className="font-semibold text-2xl">{t("home.welcome")}</h1>
          <p className="mt-1 text-muted-foreground">{t("home.welcomeDesc")}</p>
          <div className="mt-2 flex items-center gap-1 text-muted-foreground text-sm">
            <Users className="h-3.5 w-3.5" />
            <span>
              {accounts.length} {t("home.totalAccounts")}
            </span>
            <span className="mx-1">·</span>
            <Globe className="h-3.5 w-3.5" />
            <span>
              {totalDomains} {t("home.totalDomains")}
            </span>
          </div>
        </div>

        {/* 最近访问 */}
        {recentDomains.length > 0 && (
          <Card className="mb-6">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-lg">
                <Clock className="h-5 w-5" />
                {t("home.recentDomains")}
              </CardTitle>
              <CardDescription>{t("home.recentDomainsDesc")}</CardDescription>
            </CardHeader>
            <CardContent className="px-2 sm:px-6">
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
                      <div className="truncate text-muted-foreground text-xs">
                        {domain.accountName}
                      </div>
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
                    className="flex flex-col items-center gap-2 rounded-lg border p-4 text-center transition-colors hover:bg-accent"
                  >
                    <ProviderIcon provider={domain.provider} className="h-6 w-6" />
                    <div className="w-full min-w-0">
                      <div className="truncate font-medium">{domain.domainName}</div>
                      <div className="truncate text-muted-foreground text-xs">
                        {domain.accountName}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </CardContent>
          </Card>
        )}

        {/* 快捷操作 */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">{t("home.quickActions")}</CardTitle>
            <CardDescription>{t("home.quickActionsDesc")}</CardDescription>
          </CardHeader>
          <CardContent className="px-2 sm:px-6">
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
                    <div className="truncate text-muted-foreground text-xs">
                      {action.description}
                    </div>
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
                  className="flex h-auto items-center justify-start gap-3 rounded-lg border p-4 text-left transition-colors hover:bg-accent"
                >
                  <action.icon className="h-5 w-5 shrink-0" />
                  <div>
                    <div className="font-medium">{action.label}</div>
                    <div className="text-muted-foreground text-xs">{action.description}</div>
                  </div>
                </button>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </ScrollArea>
  )
}
