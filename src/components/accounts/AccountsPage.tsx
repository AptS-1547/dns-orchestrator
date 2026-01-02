import {
  Download,
  Globe,
  Loader2,
  MoreHorizontal,
  Pencil,
  Plus,
  Trash2,
  TriangleAlert,
  Upload,
  Users,
} from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { useShallow } from "zustand/react/shallow"
import { AccountForm } from "@/components/account/AccountForm"
import { ExportDialog } from "@/components/account/ExportDialog"
import { ImportDialog } from "@/components/account/ImportDialog"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
import { AccountBatchActionBar } from "@/components/accounts/AccountBatchActionBar"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { EmptyState } from "@/components/ui/empty-state"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import { useAccountStore } from "@/stores"
import type { Account } from "@/types"

export function AccountsPage() {
  const { t } = useTranslation()

  // 使用 useShallow 优化 store 订阅粒度
  const {
    accounts,
    isLoading,
    isDeleting,
    isExportDialogOpen,
    isImportDialogOpen,
    selectedAccountIds,
  } = useAccountStore(
    useShallow((state) => ({
      accounts: state.accounts,
      isLoading: state.isLoading,
      isDeleting: state.isDeleting,
      isExportDialogOpen: state.isExportDialogOpen,
      isImportDialogOpen: state.isImportDialogOpen,
      selectedAccountIds: state.selectedAccountIds,
    }))
  )

  // actions 单独获取
  const fetchAccounts = useAccountStore((state) => state.fetchAccounts)
  const deleteAccount = useAccountStore((state) => state.deleteAccount)
  const openExportDialog = useAccountStore((state) => state.openExportDialog)
  const closeExportDialog = useAccountStore((state) => state.closeExportDialog)
  const openImportDialog = useAccountStore((state) => state.openImportDialog)
  const closeImportDialog = useAccountStore((state) => state.closeImportDialog)
  const toggleAccountSelection = useAccountStore((state) => state.toggleAccountSelection)

  const isSelectMode = selectedAccountIds.size > 0

  const [showAccountForm, setShowAccountForm] = useState(false)
  const [editTarget, setEditTarget] = useState<Account | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Account | null>(null)

  const handleDelete = async () => {
    if (deleteTarget) {
      await deleteAccount(deleteTarget.id)
      setDeleteTarget(null)
    }
  }

  return (
    <PageLayout>
      <PageHeader title={t("accounts.title")} icon={<Users className="h-5 w-5" />} />

      {/* 操作栏 */}
      <div className="flex items-center justify-between border-b px-4 py-3 sm:px-6">
        <span className="text-muted-foreground text-sm">
          {t("accounts.total", { count: accounts.length })}
        </span>
        <div className="flex gap-2">
          {/* 桌面端：显示完整按钮 */}
          <div className="hidden gap-2 md:flex">
            <Button variant="outline" size="sm" onClick={openImportDialog}>
              <Upload className="mr-2 h-4 w-4" />
              {t("import.title")}
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={openExportDialog}
              disabled={accounts.length === 0}
            >
              <Download className="mr-2 h-4 w-4" />
              {t("export.title")}
            </Button>
          </div>

          {/* 移动端：收起到下拉菜单 */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild className="md:hidden">
              <Button variant="outline" size="sm">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={openImportDialog}>
                <Upload className="mr-2 h-4 w-4" />
                {t("import.title")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={openExportDialog} disabled={accounts.length === 0}>
                <Download className="mr-2 h-4 w-4" />
                {t("export.title")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>

          {/* 添加按钮始终显示 */}
          <Button size="sm" onClick={() => setShowAccountForm(true)}>
            <Plus className="h-4 w-4 md:mr-2" />
            <span className="hidden md:inline">{t("account.create")}</span>
          </Button>
        </div>
      </div>

      {/* 账户列表 */}
      <ScrollArea className="min-h-0 flex-1">
        <div className="scroll-pb-safe p-4 sm:p-6">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-20 w-full rounded-lg" />
              <Skeleton className="h-20 w-full rounded-lg" />
              <Skeleton className="h-20 w-full rounded-lg" />
            </div>
          ) : accounts.length === 0 ? (
            <EmptyState
              icon={<Globe className="h-16 w-16" />}
              title={t("accounts.empty")}
              description={t("accounts.emptyDesc")}
              size="large"
              actions={
                <>
                  <Button variant="outline" onClick={openImportDialog}>
                    <Upload className="mr-2 h-4 w-4" />
                    {t("import.title")}
                  </Button>
                  <Button onClick={() => setShowAccountForm(true)}>
                    <Plus className="mr-2 h-4 w-4" />
                    {t("account.create")}
                  </Button>
                </>
              }
            />
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {accounts.map((account) => {
                const hasError = account.status === "error"
                const isSelected = selectedAccountIds.has(account.id)
                return (
                  <Card
                    key={account.id}
                    onClick={() => toggleAccountSelection(account.id)}
                    className={cn(
                      "cursor-pointer transition-all",
                      hasError && "border-destructive/50 bg-destructive/5",
                      isSelected && "bg-primary/5 ring-2 ring-primary"
                    )}
                  >
                    <CardContent className="p-4">
                      <div className="flex items-start gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                          <ProviderIcon provider={account.provider} className="h-5 w-5" />
                        </div>
                        <div className="min-w-0 flex-1">
                          <div className="flex items-center gap-2">
                            <h3 className="truncate font-medium">{account.name}</h3>
                            {hasError && (
                              <TriangleAlert className="h-4 w-4 shrink-0 text-destructive" />
                            )}
                          </div>
                          <p className="text-muted-foreground text-sm">
                            {getProviderName(account.provider)}
                          </p>
                          {hasError && account.error && (
                            <p className="mt-1 truncate text-destructive text-xs">
                              {account.error}
                            </p>
                          )}
                        </div>
                        {!isSelectMode && (
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild onClick={(e) => e.stopPropagation()}>
                              <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                                <MoreHorizontal className="h-4 w-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem
                                onClick={(e) => {
                                  e.stopPropagation()
                                  setEditTarget(account)
                                }}
                              >
                                <Pencil className="mr-2 h-4 w-4" />
                                {t("account.editAccount")}
                              </DropdownMenuItem>
                              <DropdownMenuItem
                                onClick={(e) => {
                                  e.stopPropagation()
                                  setDeleteTarget(account)
                                }}
                                className="text-destructive focus:text-destructive"
                              >
                                <Trash2 className="mr-2 h-4 w-4" />
                                {t("account.deleteAccount")}
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                )
              })}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Dialogs */}
      <AccountForm
        open={showAccountForm || !!editTarget}
        onOpenChange={(open) => {
          if (!open) {
            setShowAccountForm(false)
            setEditTarget(null)
          }
        }}
        account={editTarget ?? undefined}
      />
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

      {/* 删除确认 */}
      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("account.deleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("account.deleteConfirmDesc", { name: deleteTarget?.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={isDeleting}>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              disabled={isDeleting}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {isDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 批量操作栏 */}
      <AccountBatchActionBar />
    </PageLayout>
  )
}
