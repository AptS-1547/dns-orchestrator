import { Loader2, MoreHorizontal, Trash2, TriangleAlert } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"
import type { Account } from "@/types"
import { getProviderName, ProviderIcon } from "./ProviderIcon"

interface AccountListProps {
  accounts: Account[]
  selectedId: string | null
  onSelect: (id: string | null) => void
  onDelete?: (id: string) => Promise<boolean>
  isDeleting?: boolean
}

export function AccountList({
  accounts,
  selectedId,
  onSelect,
  onDelete,
  isDeleting = false,
}: AccountListProps) {
  const { t } = useTranslation()
  const [deleteTarget, setDeleteTarget] = useState<Account | null>(null)

  const handleDelete = async () => {
    if (deleteTarget && onDelete) {
      await onDelete(deleteTarget.id)
      setDeleteTarget(null)
    }
  }

  if (accounts.length === 0) {
    return <div className="px-4 py-2 text-muted-foreground text-sm">{t("account.noAccounts")}</div>
  }

  return (
    <>
      <div className="space-y-1">
        {accounts.map((account) => {
          const hasError = account.status === "error"
          return (
            <div
              key={account.id}
              className={cn(
                "group flex items-center gap-2 rounded-md transition-colors",
                "hover:bg-sidebar-accent",
                selectedId === account.id && "bg-sidebar-accent",
                hasError && "opacity-60"
              )}
            >
              <button
                type="button"
                onClick={() => !hasError && onSelect(selectedId === account.id ? null : account.id)}
                disabled={hasError}
                className={cn(
                  "flex flex-1 items-center gap-3 px-3 py-2 text-left text-sm",
                  hasError && "cursor-not-allowed"
                )}
              >
                <ProviderIcon provider={account.provider} className="h-4 w-4 shrink-0" />
                <div className="flex-1 truncate">
                  <div className="truncate font-medium">{account.name}</div>
                  <div className="text-muted-foreground text-xs">
                    {getProviderName(account.provider)}
                  </div>
                </div>
                {hasError && (
                  <span title={account.error || t("account.loadFailed")}>
                    <TriangleAlert className="h-4 w-4 shrink-0 text-destructive" />
                  </span>
                )}
              </button>

              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="mr-1 h-6 w-6 opacity-0 group-hover:opacity-100"
                  >
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
          )
        })}
      </div>

      {/* 删除确认对话框 */}
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
    </>
  )
}
