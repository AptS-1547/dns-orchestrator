import { ArrowRight, Download, ExternalLink, RefreshCw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { openExternal } from "@/lib/open-external"
import { getUpdateNotes, isWebUpdate, useUpdaterStore } from "@/stores/updaterStore"
import { Button } from "./button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./dialog"
import { ScrollArea } from "./scroll-area"

interface UpdateDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function UpdateDialog({ open, onOpenChange }: UpdateDialogProps) {
  const { t } = useTranslation()
  const { available, downloading, progress, downloadAndInstall, skipVersion, maxRetries } =
    useUpdaterStore()

  if (!available) return null

  const notes = getUpdateNotes(available)
  const isWeb = isWebUpdate(available)

  const handleDownload = async () => {
    // 防止重复点击
    if (downloading) return

    // Web 端：跳转到 GitHub Release 页面
    if (isWeb) {
      await openExternal(available.url)
      onOpenChange(false)
      return
    }

    // 其他平台：下载安装
    try {
      await downloadAndInstall()
    } catch {
      toast.error(t("settings.retryFailed"), {
        description: t("settings.retryFailedDesc", { count: maxRetries }),
      })
    }
  }

  const handleSkip = () => {
    skipVersion()
    onOpenChange(false)
    toast.success(t("settings.versionSkipped", { version: available.version }))
  }

  const handleRemindLater = () => {
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md gap-3">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="h-5 w-5 text-primary" />
            {t("settings.updateDialogTitle")}
          </DialogTitle>
          <DialogDescription className="flex items-center gap-2 pt-2">
            <span className="font-mono">{__APP_VERSION__}</span>
            <ArrowRight className="h-4 w-4" />
            <span className="font-mono font-semibold text-foreground">{available.version}</span>
          </DialogDescription>
        </DialogHeader>

        {notes && (
          <div className="space-y-2">
            <p className="font-medium text-sm">{t("settings.releaseNotes")}</p>
            <ScrollArea className="h-[200px] rounded-md border bg-muted/50 p-3">
              <pre className="whitespace-pre-wrap break-all text-sm">{notes}</pre>
            </ScrollArea>
          </div>
        )}

        <DialogFooter className="flex items-center justify-between">
          {downloading ? (
            <Button disabled className="ml-auto gap-2">
              <RefreshCw className="h-4 w-4 animate-spin" />
              {t("settings.downloading")} {progress}%
            </Button>
          ) : (
            <>
              <div className="flex gap-3 text-muted-foreground text-sm">
                <button type="button" onClick={handleSkip} className="hover:underline">
                  {t("settings.skipVersion")}
                </button>
                <button type="button" onClick={handleRemindLater} className="hover:underline">
                  {t("settings.remindLater")}
                </button>
              </div>
              <Button size="sm" onClick={handleDownload} className="gap-1.5">
                {isWeb ? (
                  <>
                    <ExternalLink className="h-4 w-4" />
                    {t("settings.goToDownload")}
                  </>
                ) : (
                  <>
                    <Download className="h-4 w-4" />
                    {t("settings.updateNow")}
                  </>
                )}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
