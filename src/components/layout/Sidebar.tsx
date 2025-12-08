import { AccountTreeItem } from "@/components/navigation"
import { ProviderIcon } from "@/components/account/ProviderIcon"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"
import { useAccountStore, useDomainStore } from "@/stores"
import { Globe, Settings, Users, Wrench, X } from "lucide-react"
import { useCallback, useEffect, useState } from "react"
import { useTranslation } from "react-i18next"

interface SidebarProps {
  onOpenToolbox?: () => void
  onNavigateToMain?: () => void
  onOpenSettings?: () => void
  onOpenAccounts?: () => void
  /** 关闭 Sidebar（移动端使用） */
  onClose?: () => void
  /** 是否为移动端模式（在 Sheet 中显示） */
  isMobile?: boolean
  /** 是否折叠 */
  collapsed?: boolean
}

export function Sidebar({
  onOpenToolbox,
  onNavigateToMain,
  onOpenSettings,
  onOpenAccounts,
  onClose,
  isMobile = false,
  collapsed = false,
}: SidebarProps) {
  const { t } = useTranslation()
  const {
    accounts,
    selectedAccountId,
    isLoading: isAccountLoading,
    fetchAccounts,
    selectAccount,
  } = useAccountStore()

  const {
    domains,
    selectedDomainId,
    isLoading: isDomainLoading,
    isLoadingMore: isDomainLoadingMore,
    hasMore: hasDomainMore,
    fetchDomains,
    fetchMoreDomains,
    selectDomain,
    clearDomains,
  } = useDomainStore()

  const [expandedAccountId, setExpandedAccountId] = useState<string | null>(null)

  // 展开/收起账户
  const handleToggleAccount = useCallback(
    (accountId: string) => {
      if (expandedAccountId === accountId) {
        setExpandedAccountId(null)
        selectAccount(null)
        clearDomains()
      } else {
        setExpandedAccountId(accountId)
        selectAccount(accountId)
        fetchDomains(accountId)
      }
    },
    [expandedAccountId, selectAccount, fetchDomains, clearDomains]
  )

  // 选择域名
  const handleSelectDomain = useCallback(
    (domainId: string) => {
      selectDomain(domainId)
      onNavigateToMain?.()
      if (isMobile) {
        onClose?.()
      }
    },
    [selectDomain, onNavigateToMain, isMobile, onClose]
  )

  // 初始化加载账户列表
  useEffect(() => {
    fetchAccounts()
  }, [fetchAccounts])

  // 恢复展开状态
  useEffect(() => {
    if (selectedAccountId && !expandedAccountId) {
      setExpandedAccountId(selectedAccountId)
    }
  }, [selectedAccountId, expandedAccountId])

  return (
    <TooltipProvider delayDuration={300}>
      <aside
        className={cn(
          "flex h-full flex-col border-r bg-sidebar transition-all duration-200",
          isMobile ? "w-full" : collapsed ? "w-16" : "w-64"
        )}
      >
        {/* Header */}
        <div
          className={cn(
            "flex items-center border-b",
            collapsed ? "justify-center p-3" : "justify-between p-4"
          )}
        >
          <div className="flex items-center gap-2">
            <Globe className="h-6 w-6 text-primary shrink-0" />
            {!collapsed && (
              <h1 className="font-semibold text-lg whitespace-nowrap">{t("common.appName")}</h1>
            )}
          </div>
          {isMobile && onClose && (
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-5 w-5" />
            </Button>
          )}
        </div>

        {/* 账户标题 - 折叠时隐藏 */}
        {!collapsed && (
          <div className="flex items-center justify-between border-b px-4 py-2">
            <span className="text-muted-foreground text-sm font-medium">{t("account.title")}</span>
            <Button
              variant="ghost"
              size="sm"
              className="h-7 px-2 text-muted-foreground"
              onClick={() => {
                onOpenAccounts?.()
                if (isMobile) onClose?.()
              }}
            >
              <Users className="h-4 w-4" />
            </Button>
          </div>
        )}

        {/* 账户树形列表 */}
        <ScrollArea className="flex-1">
          <div className={cn("space-y-2", collapsed ? "p-2" : "p-3", isMobile && "p-2")}>
            {isAccountLoading ? (
              <div className="space-y-2">
                <Skeleton className={cn("rounded-lg", collapsed ? "h-10 w-10" : "h-14 w-full")} />
                <Skeleton className={cn("rounded-lg", collapsed ? "h-10 w-10" : "h-14 w-full")} />
              </div>
            ) : accounts.length === 0 ? (
              collapsed ? (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="w-full h-10"
                      onClick={() => {
                        onOpenAccounts?.()
                      }}
                    >
                      <Users className="h-5 w-5 text-muted-foreground" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent side="right">{t("accounts.manage")}</TooltipContent>
                </Tooltip>
              ) : (
                <div className="flex flex-col items-center justify-center py-6 text-center">
                  <Globe className="h-8 w-8 text-muted-foreground/50 mb-2" />
                  <p className="text-muted-foreground text-sm mb-3">{t("account.noAccounts")}</p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => {
                      onOpenAccounts?.()
                      if (isMobile) onClose?.()
                    }}
                  >
                    <Users className="h-4 w-4 mr-1" />
                    {t("accounts.manage")}
                  </Button>
                </div>
              )
            ) : collapsed ? (
              // 折叠模式：只显示账户图标
              accounts.map((account) => (
                <Tooltip key={account.id}>
                  <TooltipTrigger asChild>
                    <div
                      className={cn(
                        "flex w-full items-center justify-center rounded-lg p-2 transition-colors",
                        "hover:bg-sidebar-accent",
                        selectedAccountId === account.id && "bg-sidebar-accent"
                      )}
                    >
                      <ProviderIcon provider={account.provider} className="h-5 w-5" />
                    </div>
                  </TooltipTrigger>
                  <TooltipContent side="right">{account.name}</TooltipContent>
                </Tooltip>
              ))
            ) : (
              // 展开模式：显示完整树形结构
              accounts.map((account) => (
                <AccountTreeItem
                  key={account.id}
                  account={account}
                  domains={expandedAccountId === account.id ? domains : []}
                  isExpanded={expandedAccountId === account.id}
                  selectedDomainId={selectedDomainId}
                  onToggle={() => handleToggleAccount(account.id)}
                  onSelectDomain={handleSelectDomain}
                  onDelete={() => {
                    // 跳转到账户管理页面进行删除
                    onOpenAccounts?.()
                    if (isMobile) onClose?.()
                  }}
                  isLoading={expandedAccountId === account.id && isDomainLoading}
                  isLoadingMore={expandedAccountId === account.id && isDomainLoadingMore}
                  hasMore={expandedAccountId === account.id && hasDomainMore}
                  onLoadMore={() => fetchMoreDomains(account.id)}
                />
              ))
            )}
          </div>
        </ScrollArea>

        {/* 底部导航 */}
        <div className={cn("border-t p-2 space-y-1", collapsed && "flex flex-col items-center")}>
          {collapsed ? (
            <>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-10 w-10"
                    onClick={() => {
                      onOpenToolbox?.()
                    }}
                  >
                    <Wrench className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="right">{t("toolbox.title")}</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-10 w-10"
                    onClick={() => {
                      onOpenSettings?.()
                    }}
                  >
                    <Settings className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent side="right">{t("settings.title")}</TooltipContent>
              </Tooltip>
            </>
          ) : (
            <>
              <Button
                variant="ghost"
                className="w-full justify-start gap-3 h-10"
                onClick={() => {
                  onOpenToolbox?.()
                  if (isMobile) onClose?.()
                }}
              >
                <Wrench className="h-4 w-4" />
                <span>{t("toolbox.title")}</span>
              </Button>
              <Button
                variant="ghost"
                className="w-full justify-start gap-3 h-10"
                onClick={() => {
                  onOpenSettings?.()
                  if (isMobile) onClose?.()
                }}
              >
                <Settings className="h-4 w-4" />
                <span>{t("settings.title")}</span>
              </Button>
            </>
          )}
        </div>
      </aside>
    </TooltipProvider>
  )
}
