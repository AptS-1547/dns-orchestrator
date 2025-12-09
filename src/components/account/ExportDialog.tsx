import { save } from "@tauri-apps/plugin-dialog"
import { writeTextFile } from "@tauri-apps/plugin-fs"
import { Download, Loader2, Lock } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { getErrorMessage } from "@/lib/error"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Switch } from "@/components/ui/switch"
import { invoke } from "@/lib/tauri"
import type { Account, ExportAccountsRequest } from "@/types"
import { getProviderName, ProviderIcon } from "./ProviderIcon"

interface ExportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  accounts: Account[]
}

export function ExportDialog({ open, onOpenChange, accounts }: ExportDialogProps) {
  const { t } = useTranslation()
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [encrypt, setEncrypt] = useState(false)
  const [password, setPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [isExporting, setIsExporting] = useState(false)

  // 全选/取消全选
  const allSelected = selectedIds.size === accounts.length && accounts.length > 0

  const handleSelectAll = () => {
    if (allSelected) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(accounts.map((a) => a.id)))
    }
  }

  const handleToggle = (id: string) => {
    const newSet = new Set(selectedIds)
    if (newSet.has(id)) {
      newSet.delete(id)
    } else {
      newSet.add(id)
    }
    setSelectedIds(newSet)
  }

  const validateExportForm = (): boolean => {
    if (selectedIds.size === 0) {
      toast.error(t("export.noAccountsSelected"))
      return false
    }
    if (!encrypt) return true

    if (!password) {
      toast.error(t("export.passwordRequired"))
      return false
    }
    if (password !== confirmPassword) {
      toast.error(t("export.passwordMismatch"))
      return false
    }
    if (password.length < 6) {
      toast.error(t("export.passwordTooShort"))
      return false
    }
    return true
  }

  const resetForm = () => {
    setSelectedIds(new Set())
    setEncrypt(false)
    setPassword("")
    setConfirmPassword("")
  }

  const handleExport = async () => {
    if (!validateExportForm()) return

    setIsExporting(true)
    try {
      const request: ExportAccountsRequest = {
        accountIds: Array.from(selectedIds),
        encrypt,
        password: encrypt ? password : undefined,
      }

      const response = await invoke("export_accounts", { request })
      if (!(response.success && response.data)) {
        toast.error(getErrorMessage(response.error))
        return
      }

      const filePath = await save({
        defaultPath: response.data.suggestedFilename,
        filters: [{ name: "DNS Orchestrator Backup", extensions: ["dnso"] }],
      })
      if (!filePath) return

      await writeTextFile(filePath, response.data.content)
      toast.success(t("export.success", { count: selectedIds.size }))
      onOpenChange(false)
      resetForm()
    } catch (err) {
      toast.error(String(err))
    } finally {
      setIsExporting(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{t("export.title")}</DialogTitle>
          <DialogDescription>{t("export.description")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* 账号选择列表 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>{t("export.selectAccounts")}</Label>
              <Button variant="ghost" size="sm" onClick={handleSelectAll}>
                {allSelected ? t("common.deselectAll") : t("common.selectAll")}
              </Button>
            </div>
            <ScrollArea className="h-[200px] rounded-md border p-3">
              <div className="space-y-2">
                {accounts.map((account) => (
                  <div key={account.id} className="flex items-center gap-3 py-1">
                    <Checkbox
                      checked={selectedIds.has(account.id)}
                      onCheckedChange={() => handleToggle(account.id)}
                      disabled={account.status === "error"}
                    />
                    <ProviderIcon provider={account.provider} className="h-4 w-4" />
                    <span className="flex-1 truncate text-sm">{account.name}</span>
                    <span className="text-muted-foreground text-xs">
                      {getProviderName(account.provider)}
                    </span>
                  </div>
                ))}
              </div>
            </ScrollArea>
            <p className="text-muted-foreground text-xs">
              {t("export.selectedCount", { count: selectedIds.size })}
            </p>
          </div>

          {/* 加密选项 */}
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Lock className="h-4 w-4" />
                <Label htmlFor="encrypt">{t("export.encryptExport")}</Label>
              </div>
              <Switch id="encrypt" checked={encrypt} onCheckedChange={setEncrypt} />
            </div>

            {encrypt && (
              <div className="space-y-3 pl-6">
                <div className="space-y-2">
                  <Label htmlFor="password">{t("export.password")}</Label>
                  <Input
                    id="password"
                    type="password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder={t("export.passwordPlaceholder")}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="confirmPassword">{t("export.confirmPassword")}</Label>
                  <Input
                    id="confirmPassword"
                    type="password"
                    value={confirmPassword}
                    onChange={(e) => setConfirmPassword(e.target.value)}
                    placeholder={t("export.confirmPasswordPlaceholder")}
                  />
                </div>
                <p className="text-muted-foreground text-xs">{t("export.encryptionNote")}</p>
              </div>
            )}
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button onClick={handleExport} disabled={isExporting || selectedIds.size === 0}>
            {isExporting ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Download className="mr-2 h-4 w-4" />
            )}
            {t("export.exportButton")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
