import { Check, Download, RefreshCw, X } from "lucide-react"
import { useEffect } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { EXTERNAL_LINKS } from "@/constants"
import { logger } from "@/lib/logger"
import { openExternal } from "@/lib/open-external"
import { getUpdateNotes, useUpdaterStore } from "@/stores/updaterStore"

interface UpdateCheckerProps {
  /** 是否自动重置 upToDate 状态，默认 true */
  autoReset?: boolean
}

/**
 * 更新检查组件
 * 负责检查应用更新、下载安装、显示发行说明等
 */
export function UpdateChecker({ autoReset = true }: UpdateCheckerProps) {
  const { t } = useTranslation()
  const {
    checking,
    downloading,
    progress,
    available,
    upToDate,
    error,
    isPlatformUnsupported,
    checkForUpdates,
    downloadAndInstall,
    skipVersion,
    resetUpToDate,
  } = useUpdaterStore()

  // 组件加载时重置 upToDate 状态，允许用户再次检查更新
  useEffect(() => {
    if (autoReset) {
      resetUpToDate()
    }
  }, [autoReset, resetUpToDate])

  // 跳过版本处理
  const handleSkipVersion = () => {
    if (available) {
      skipVersion()
      toast.success(t("settings.versionSkipped", { version: available.version }))
    }
  }

  // 手动检查更新处理
  const handleCheckUpdates = async () => {
    if (checking) return
    try {
      const update = await checkForUpdates()
      // 如果有错误（平台不支持），显示错误提示
      if (!update) {
        const { error: checkError, isPlatformUnsupported: platformError } =
          useUpdaterStore.getState()
        if (checkError) {
          if (platformError) {
            toast.error(t("settings.platformNotSupported"), {
              description: t("settings.platformNotSupportedDesc"),
              action: {
                label: "GitHub Releases",
                onClick: async () => {
                  try {
                    await openExternal(EXTERNAL_LINKS.GITHUB_RELEASES)
                  } catch (err) {
                    logger.error("Failed to open URL:", err)
                  }
                },
              },
            })
          } else {
            toast.error(t("settings.updateCheckError"), {
              description: t("settings.updateCheckErrorDesc", { error: checkError }),
            })
          }
        }
      }
    } catch (error) {
      // 异常情况
      const errorMsg = error instanceof Error ? error.message : String(error)
      toast.error(t("settings.updateCheckError"), {
        description: t("settings.updateCheckErrorDesc", { error: errorMsg }),
      })
    }
  }

  // 下载并安装处理
  const handleDownloadAndInstall = async () => {
    if (downloading) return
    try {
      await downloadAndInstall()
      // 下载完成后应用会重启
    } catch {
      const { maxRetries } = useUpdaterStore.getState()
      toast.error(t("settings.retryFailed"), {
        description: t("settings.retryFailedDesc", { count: maxRetries }),
      })
    }
  }

  return (
    <div className="flex flex-col gap-3 pt-3">
      {/* 更新按钮区域 */}
      <div className="flex items-center gap-3">
        {available ? (
          <>
            {/* 有更新可用：下载按钮 + 跳过按钮 */}
            <Button
              size="sm"
              onClick={handleDownloadAndInstall}
              disabled={downloading}
              className="gap-2"
            >
              {downloading ? (
                <>
                  <RefreshCw className="h-4 w-4 animate-spin" />
                  {t("settings.downloading")} {progress}%
                </>
              ) : (
                <>
                  <Download className="h-4 w-4" />
                  {t("settings.updateNow")} ({available.version})
                </>
              )}
            </Button>
            {!downloading && (
              <Button variant="outline" size="sm" onClick={handleSkipVersion} className="gap-2">
                <X className="h-4 w-4" />
                {t("settings.skipVersion")}
              </Button>
            )}
          </>
        ) : (
          /* 无更新或未检查：检查更新按钮 */
          <Button
            variant={upToDate ? "default" : "outline"}
            size="sm"
            onClick={handleCheckUpdates}
            disabled={checking || upToDate}
            className="gap-2"
          >
            {checking ? (
              <>
                <RefreshCw className="h-4 w-4 animate-spin" />
                {t("settings.checking")}
              </>
            ) : upToDate ? (
              <>
                <Check className="h-4 w-4" />
                {t("settings.noUpdate")}
              </>
            ) : (
              <>
                <Check className="h-4 w-4" />
                {t("settings.checkUpdate")}
              </>
            )}
          </Button>
        )}
      </div>

      {/* 错误提示 */}
      {error && (
        <p className="text-destructive text-xs">
          {isPlatformUnsupported
            ? t("settings.platformNotSupported")
            : t("settings.updateCheckError")}
        </p>
      )}

      {/* 发行说明 */}
      {available && getUpdateNotes(available) && (
        <div className="space-y-2 border-t pt-3">
          <p className="font-medium text-sm">{t("settings.releaseNotes")}</p>
          <ScrollArea className="h-[150px] rounded-md border bg-muted/50 p-3">
            <pre className="whitespace-pre-wrap break-all text-xs">{getUpdateNotes(available)}</pre>
          </ScrollArea>
        </div>
      )}
    </div>
  )
}
