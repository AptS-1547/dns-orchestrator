import { open as openFileDialog } from "@tauri-apps/plugin-dialog"
import { readTextFile } from "@tauri-apps/plugin-fs"
import { AlertTriangle, FileText, Loader2, Lock, Upload } from "lucide-react"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
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
import { getErrorMessage } from "@/lib/error"
import { accountService } from "@/services"
import type { ImportAccountsRequest, ImportPreview } from "@/types"
import { getProviderName, ProviderIcon } from "./ProviderIcon"

interface ImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onImportSuccess: () => void
}

type ImportStep = "select" | "password" | "preview" | "importing"

export function ImportDialog({ open, onOpenChange, onImportSuccess }: ImportDialogProps) {
  const { t } = useTranslation()
  const [step, setStep] = useState<ImportStep>("select")
  const [fileContent, setFileContent] = useState<string | null>(null)
  const [fileName, setFileName] = useState<string | null>(null)
  const [password, setPassword] = useState("")
  const [preview, setPreview] = useState<ImportPreview | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const resetState = useCallback(() => {
    setStep("select")
    setFileContent(null)
    setFileName(null)
    setPassword("")
    setPreview(null)
    setIsLoading(false)
  }, [])

  const handleSelectFile = async () => {
    try {
      const filePath = await openFileDialog({
        multiple: false,
        filters: [{ name: "DNS Orchestrator Backup", extensions: ["dnso"] }],
      })

      if (!filePath) return

      setIsLoading(true)
      const content = await readTextFile(filePath as string)
      setFileContent(content)
      setFileName((filePath as string).split("/").pop() || "file.json")

      // 尝试预览（不带密码）
      const response = await accountService.previewImport(content, null)

      if (response.success && response.data) {
        setPreview(response.data)
        if (response.data.encrypted && !response.data.accounts) {
          setStep("password")
        } else {
          setStep("preview")
        }
      } else {
        toast.error(getErrorMessage(response.error))
      }
    } catch (err) {
      toast.error(String(err))
    } finally {
      setIsLoading(false)
    }
  }

  const handleDecrypt = async () => {
    if (!(fileContent && password)) return

    setIsLoading(true)
    try {
      const response = await accountService.previewImport(fileContent, password)

      if (response.success && response.data?.accounts) {
        setPreview(response.data)
        setStep("preview")
      } else {
        toast.error(getErrorMessage(response.error))
      }
    } catch (err) {
      toast.error(String(err))
    } finally {
      setIsLoading(false)
    }
  }

  const showImportResult = (successCount: number, failures: { name: string; reason: string }[]) => {
    if (successCount > 0) {
      toast.success(t("import.success", { count: successCount }))
      onImportSuccess()
    }
    if (failures.length > 0) {
      toast.warning(t("import.partialFailure", { count: failures.length }), {
        description: failures.map((f) => `${f.name}: ${f.reason}`).join("\n"),
      })
    }
  }

  const handleImport = async () => {
    if (!fileContent) return

    setStep("importing")
    try {
      const request: ImportAccountsRequest = {
        content: fileContent,
        password: preview?.encrypted ? password : undefined,
      }

      const response = await accountService.importAccounts(request)

      if (response.success && response.data) {
        showImportResult(response.data.successCount, response.data.failures)
        onOpenChange(false)
        resetState()
      } else {
        toast.error(getErrorMessage(response.error))
        setStep("preview")
      }
    } catch (err) {
      toast.error(String(err))
      setStep("preview")
    }
  }

  const handleClose = () => {
    onOpenChange(false)
    resetState()
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{t("import.title")}</DialogTitle>
          <DialogDescription>
            {step === "select" && t("import.selectFileDesc")}
            {step === "password" && t("import.enterPasswordDesc")}
            {step === "preview" && t("import.previewDesc")}
            {step === "importing" && t("import.importingDesc")}
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          {/* Step 1: 选择文件 */}
          {step === "select" && (
            <div className="flex flex-col items-center justify-center gap-4 py-8">
              <div className="rounded-full bg-muted p-4">
                <Upload className="h-8 w-8 text-muted-foreground" />
              </div>
              <Button onClick={handleSelectFile} disabled={isLoading}>
                {isLoading ? (
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                ) : (
                  <FileText className="mr-2 h-4 w-4" />
                )}
                {t("import.selectFile")}
              </Button>
            </div>
          )}

          {/* Step 2: 输入密码 */}
          {step === "password" && (
            <div className="space-y-4">
              <div className="flex items-center gap-2 rounded-md bg-muted p-3">
                <Lock className="h-4 w-4" />
                <span className="text-sm">{t("import.fileEncrypted")}</span>
              </div>
              <div className="space-y-2">
                <Label htmlFor="import-password">{t("import.password")}</Label>
                <Input
                  id="import-password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder={t("import.passwordPlaceholder")}
                  onKeyDown={(e) => e.key === "Enter" && handleDecrypt()}
                />
              </div>
            </div>
          )}

          {/* Step 3: 预览 */}
          {step === "preview" && preview?.accounts && (
            <div className="space-y-4">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">{fileName}</span>
                <Badge variant="secondary">
                  {t("import.accountCount", { count: preview.accountCount })}
                </Badge>
              </div>
              <ScrollArea className="h-[200px] rounded-md border p-3">
                <div className="space-y-2">
                  {preview.accounts.map((account) => (
                    <div
                      key={`${account.name}-${account.provider}`}
                      className="flex items-center gap-3 py-1"
                    >
                      <ProviderIcon provider={account.provider} className="h-4 w-4" />
                      <span className="flex-1 truncate text-sm">{account.name}</span>
                      <span className="text-muted-foreground text-xs">
                        {getProviderName(account.provider)}
                      </span>
                      {account.hasConflict && (
                        <span title={t("import.nameConflict")}>
                          <AlertTriangle className="h-4 w-4 text-warning" />
                        </span>
                      )}
                    </div>
                  ))}
                </div>
              </ScrollArea>
              {preview.accounts.some((a) => a.hasConflict) && (
                <p className="flex items-center gap-1 text-muted-foreground text-xs">
                  <AlertTriangle className="h-3 w-3" />
                  {t("import.conflictNote")}
                </p>
              )}
            </div>
          )}

          {/* Step 4: 导入中 */}
          {step === "importing" && (
            <div className="flex flex-col items-center justify-center gap-4 py-8">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
              <p className="text-muted-foreground text-sm">{t("import.importing")}</p>
            </div>
          )}
        </div>

        <DialogFooter>
          {step !== "importing" && (
            <Button variant="outline" onClick={handleClose}>
              {t("common.cancel")}
            </Button>
          )}
          {step === "password" && (
            <Button onClick={handleDecrypt} disabled={!password || isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {t("import.decrypt")}
            </Button>
          )}
          {step === "preview" && (
            <Button onClick={handleImport}>
              <Upload className="mr-2 h-4 w-4" />
              {t("import.importButton")}
            </Button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
