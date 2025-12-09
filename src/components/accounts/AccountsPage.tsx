import {
  ArrowLeft,
  Download,
  Globe,
  Loader2,
  MoreHorizontal,
  Plus,
  Trash2,
  TriangleAlert,
  Upload,
  Users,
} from "lucide-react"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { AccountForm } from "@/components/account/AccountForm"
import { ExportDialog } from "@/components/account/ExportDialog"
import { ImportDialog } from "@/components/account/ImportDialog"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
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
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { useAccountStore } from "@/stores"
import type { Account } from "@/types"

interface AccountsPageProps {
  onBack: () => void
}

export function AccountsPage({ onBack }: AccountsPageProps) {
  const { t } = useTranslation()
  const {
    accounts,
    isLoading,
    isDeleting,
    fetchAccounts,
    deleteAccount,
    isExportDialogOpen,
    isImportDialogOpen,
    openExportDialog,
    closeExportDialog,
    openImportDialog,
    closeImportDialog,
  } = useAccountStore()

  const [showAccountForm, setShowAccountForm] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<Account | null>(null)

  useEffect(() => {
    fetchAccounts()
  }, [fetchAccounts])

  const handleDelete = async () => {
    if (deleteTarget) {
      await deleteAccount(deleteTarget.id)
      setDeleteTarget(null)
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 border-b bg-background px-4 py-3 sm:gap-4 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex items-center gap-2">
          <Users className="h-5 w-5 text-primary" />
          <h2 className="font-semibold text-xl">{t("accounts.title")}</h2>
        </div>
      </div>

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
        <div className="p-4 sm:p-6">
          {isLoading ? (
            <div className="space-y-3">
              <Skeleton className="h-20 w-full rounded-lg" />
              <Skeleton className="h-20 w-full rounded-lg" />
              <Skeleton className="h-20 w-full rounded-lg" />
            </div>
          ) : accounts.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-16 text-center">
              <Globe className="mb-4 h-16 w-16 text-muted-foreground/30" />
              <h3 className="mb-2 font-medium text-lg">{t("accounts.empty")}</h3>
              <p className="mb-6 max-w-sm text-muted-foreground text-sm">
                {t("accounts.emptyDesc")}
              </p>
              <div className="flex gap-3">
                <Button variant="outline" onClick={openImportDialog}>
                  <Upload className="mr-2 h-4 w-4" />
                  {t("import.title")}
                </Button>
                <Button onClick={() => setShowAccountForm(true)}>
                  <Plus className="mr-2 h-4 w-4" />
                  {t("account.create")}
                </Button>
              </div>
            </div>
          ) : (
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {accounts.map((account) => {
                const hasError = account.status === "error"
                return (
                  <Card
                    key={account.id}
                    className={hasError ? "border-destructive/50 bg-destructive/5" : ""}
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
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                              <MoreHorizontal className="h-4 w-4" />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem
                              onClick={() => setDeleteTarget(account)}
                              className="text-destructive focus:text-destructive"
                            >
                              <Trash2 className="mr-2 h-4 w-4" />
                              {t("account.deleteAccount")}
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
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
    </div>
  )
}
