import {
  ArrowLeft,
  Check,
  Download,
  Github,
  Languages,
  Monitor,
  Moon,
  RefreshCw,
  Sun,
  X,
} from "lucide-react"
import { useEffect } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Switch } from "@/components/ui/switch"
import { EXTERNAL_LINKS } from "@/constants"
import { type LanguageCode, supportedLanguages } from "@/i18n"
import { ENV } from "@/lib/env"
import { logger } from "@/lib/logger"
import { openExternal } from "@/lib/open-external"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores/settingsStore"
import { getUpdateNotes, useUpdaterStore } from "@/stores/updaterStore"

export function SettingsPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { theme, language, debugMode, setTheme, setLanguage, setDebugMode } = useSettingsStore()
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

  // 每次进入设置页面时，重置 upToDate 状态，允许用户再次检查更新
  useEffect(() => {
    resetUpToDate()
  }, [resetUpToDate])

  // 跳过版本处理
  const handleSkipVersion = () => {
    if (available) {
      skipVersion()
      toast.success(t("settings.versionSkipped", { version: available.version }))
    }
  }

  // 手动检查更新处理
  const handleCheckUpdates = async () => {
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

  const themes = [
    { id: "light" as const, label: t("settings.themeLight"), icon: Sun },
    { id: "dark" as const, label: t("settings.themeDark"), icon: Moon },
    { id: "system" as const, label: t("settings.themeSystem"), icon: Monitor },
  ]

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b bg-background px-4 py-3 sm:gap-4 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={() => navigate("/")}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <h2 className="font-semibold text-xl">{t("settings.title")}</h2>
      </div>

      {/* Content */}
      <ScrollArea className="min-h-0 flex-1">
        <div className="mx-auto max-w-3xl space-y-6 p-4 sm:space-y-10 sm:p-8">
          {/* 主题设置 */}
          <div className="space-y-3 sm:space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.appearance")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.theme")}</p>
            </div>
            <div className="grid grid-cols-3 gap-2 sm:gap-4">
              {themes.map(({ id, label, icon: Icon }) => (
                <button
                  key={id}
                  type="button"
                  onClick={() => setTheme(id)}
                  className={cn(
                    "flex flex-col items-center gap-2 rounded-xl border-2 p-3 transition-all sm:gap-3 sm:p-5",
                    theme === id
                      ? "border-primary bg-primary/5 shadow-sm"
                      : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
                  )}
                >
                  <Icon className="h-5 w-5 sm:h-7 sm:w-7" />
                  <span className="whitespace-nowrap font-medium text-xs sm:text-sm">{label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 语言设置 */}
          <div className="space-y-3 sm:space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.language")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.languageDesc")}</p>
            </div>
            <div className="grid grid-cols-2 gap-2 sm:gap-4">
              {supportedLanguages.map((lang) => (
                <button
                  key={lang.code}
                  type="button"
                  onClick={() => setLanguage(lang.code as LanguageCode)}
                  className={cn(
                    "flex items-center gap-2 rounded-xl border-2 p-3 transition-all sm:gap-3 sm:p-4",
                    language === lang.code
                      ? "border-primary bg-primary/5 shadow-sm"
                      : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
                  )}
                >
                  <Languages className="h-4 w-4 sm:h-5 sm:w-5" />
                  <span className="font-medium text-sm">{lang.name}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 通知设置 */}
          <div className="space-y-3 sm:space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.notifications")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.notificationsDesc")}</p>
            </div>
            <div className="flex items-center justify-between rounded-xl border bg-card p-4 sm:p-5">
              <div className="space-y-1.5">
                <Label htmlFor="notifications" className="font-medium text-sm">
                  {t("settings.operationNotifications")}
                </Label>
                <p className="text-muted-foreground text-xs">
                  {t("settings.operationNotificationsDesc")}
                </p>
              </div>
              <Switch id="notifications" defaultChecked />
            </div>
          </div>

          {/* 调试模式设置 - 仅开发环境显示 */}
          {ENV.isDev && (
            <div className="space-y-3 sm:space-y-5">
              <div>
                <h3 className="mb-1 font-semibold text-lg">{t("settings.debug")}</h3>
                <p className="text-muted-foreground text-sm">{t("settings.debugDesc")}</p>
              </div>
              <div className="flex items-center justify-between rounded-xl border bg-card p-4 sm:p-5">
                <div className="space-y-1.5">
                  <Label htmlFor="debug-mode" className="font-medium text-sm">
                    {t("settings.debugMode")}
                  </Label>
                  <p className="text-muted-foreground text-xs">{t("settings.debugModeDesc")}</p>
                </div>
                <Switch id="debug-mode" checked={debugMode} onCheckedChange={setDebugMode} />
              </div>
            </div>
          )}

          {/* 关于 */}
          <div className="space-y-3 sm:space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.about")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.aboutDesc")}</p>
            </div>
            <div className="space-y-4 rounded-xl border bg-card p-4 sm:space-y-5 sm:p-5">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <p className="font-medium">{t("common.appName")}</p>
                  <p className="text-muted-foreground text-sm">
                    {t("settings.version")} {ENV.appVersion}
                  </p>
                </div>
                <Button
                  variant="outline"
                  size="icon"
                  onClick={() => openExternal(EXTERNAL_LINKS.GITHUB_REPO)}
                >
                  <Github className="h-4 w-4" />
                </Button>
              </div>

              {/* 检查更新 */}
              <div className="flex flex-col gap-3 border-t pt-2">
                <div className="flex items-center gap-3 pt-3">
                  {available ? (
                    <>
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
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={handleSkipVersion}
                          className="gap-2"
                        >
                          <X className="h-4 w-4" />
                          {t("settings.skipVersion")}
                        </Button>
                      )}
                    </>
                  ) : (
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
                      <pre className="whitespace-pre-wrap text-xs">{getUpdateNotes(available)}</pre>
                    </ScrollArea>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
