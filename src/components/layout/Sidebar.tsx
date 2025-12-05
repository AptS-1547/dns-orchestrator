import { AccountForm } from "@/components/account/AccountForm"
import { AccountList } from "@/components/account/AccountList"
import { ExportDialog } from "@/components/account/ExportDialog"
import { ImportDialog } from "@/components/account/ImportDialog"
import { DomainList } from "@/components/domain/DomainList"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { Skeleton } from "@/components/ui/skeleton"
import { useAccountStore, useDomainStore } from "@/stores"
import { Download, Globe, Plus, Upload, Wrench } from "lucide-react"
import { useCallback, useEffect, useState } from "react"
import { useTranslation } from "react-i18next"

interface SidebarProps {
  onOpenToolbox?: () => void
  onNavigateToMain?: () => void
}

export function Sidebar({ onOpenToolbox, onNavigateToMain }: SidebarProps) {
  const { t } = useTranslation()
  const {
    accounts,
    selectedAccountId,
    isLoading: isAccountLoading,
    isDeleting: isAccountDeleting,
    fetchAccounts,
    selectAccount,
    deleteAccount,
    isExportDialogOpen,
    isImportDialogOpen,
    openExportDialog,
    closeExportDialog,
    openImportDialog,
    closeImportDialog,
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

  const [showAccountForm, setShowAccountForm] = useState(false)

  // 切换账户时先清除域名选择，避免用旧的 domainId 请求新账户
  const handleSelectAccount = useCallback(
    (id: string | null) => {
      if (id !== selectedAccountId) {
        selectDomain(null)
      }
      selectAccount(id)
    },
    [selectedAccountId, selectAccount, selectDomain]
  )

  useEffect(() => {
    fetchAccounts()
  }, [fetchAccounts])

  useEffect(() => {
    if (selectedAccountId) {
      fetchDomains(selectedAccountId)
    } else {
      clearDomains()
    }
  }, [selectedAccountId, fetchDomains, clearDomains])

  return (
    <aside className="flex w-64 flex-col border-r bg-sidebar">
      {/* Header */}
      <div className="border-b p-4">
        <div className="flex items-center gap-2">
          <Globe className="h-6 w-6 text-primary" />
          <h1 className="font-semibold text-lg">{t("common.appName")}</h1>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {/* 账号列表 */}
        <div className="p-4">
          <div className="mb-3 flex items-center justify-between">
            <h2 className="font-medium text-muted-foreground text-sm">{t("account.title")}</h2>
            <div className="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={openExportDialog}
                disabled={accounts.length === 0}
                title={t("export.title")}
              >
                <Download className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={openImportDialog}
                title={t("import.title")}
              >
                <Upload className="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6"
                onClick={() => setShowAccountForm(true)}
                title={t("account.create")}
              >
                <Plus className="h-4 w-4" />
              </Button>
            </div>
          </div>
          {isAccountLoading ? (
            <div className="space-y-2">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ) : (
            <AccountList
              accounts={accounts}
              selectedId={selectedAccountId}
              onSelect={handleSelectAccount}
              onDelete={deleteAccount}
              isDeleting={isAccountDeleting}
            />
          )}
        </div>

        {/* 域名列表 */}
        {selectedAccountId && (
          <>
            <Separator />
            <div className="p-4">
              <h2 className="mb-3 font-medium text-muted-foreground text-sm">
                {t("domain.title")}
              </h2>
              {isDomainLoading ? (
                <div className="space-y-2">
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                </div>
              ) : (
                <DomainList
                  domains={domains}
                  selectedId={selectedDomainId}
                  onSelect={(id) => {
                    selectDomain(id)
                    onNavigateToMain?.()
                  }}
                  hasMore={hasDomainMore}
                  isLoadingMore={isDomainLoadingMore}
                  onLoadMore={() => selectedAccountId && fetchMoreDomains(selectedAccountId)}
                />
              )}
            </div>
          </>
        )}
      </ScrollArea>

      {/* 底部工具箱按钮 */}
      <Separator />
      <div className="p-3">
        <Button variant="ghost" className="w-full justify-start gap-2" onClick={onOpenToolbox}>
          <Wrench className="h-4 w-4" />
          {t("toolbox.title")}
        </Button>
      </div>

      {/* Dialogs */}
      <AccountForm open={showAccountForm} onOpenChange={setShowAccountForm} />
      <ExportDialog
        open={isExportDialogOpen}
        onOpenChange={closeExportDialog}
        accounts={accounts}
      />
      <ImportDialog
        open={isImportDialogOpen}
        onOpenChange={closeImportDialog}
        onImportSuccess={fetchAccounts}
      />
    </aside>
  )
}
