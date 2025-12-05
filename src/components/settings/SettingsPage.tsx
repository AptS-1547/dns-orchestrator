import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Switch } from "@/components/ui/switch"
import { type LanguageCode, supportedLanguages } from "@/i18n"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores/settingsStore"
import { useUpdaterStore } from "@/stores/updaterStore"
import { openUrl } from "@tauri-apps/plugin-opener"
import {
  ArrowLeft,
  Check,
  Download,
  Languages,
  Monitor,
  Moon,
  RefreshCw,
  Sun,
  X,
} from "lucide-react"
import { useEffect } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"

interface SettingsPageProps {
  onBack: () => void
}

export function SettingsPage({ onBack }: SettingsPageProps) {
  const { t } = useTranslation()
  const { theme, language, setTheme, setLanguage } = useSettingsStore()
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
                    await openUrl("https://github.com/AptS-1547/dns-orchestrator/releases/latest")
                  } catch (err) {
                    console.error("Failed to open URL:", err)
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
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-4 border-b bg-background px-6 py-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <h2 className="font-semibold text-xl">{t("settings.title")}</h2>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="mx-auto max-w-3xl space-y-10 p-8">
          {/* 主题设置 */}
          <div className="space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.appearance")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.theme")}</p>
            </div>
            <div className="grid grid-cols-3 gap-4">
              {themes.map(({ id, label, icon: Icon }) => (
                <button
                  key={id}
                  type="button"
                  onClick={() => setTheme(id)}
                  className={cn(
                    "flex flex-col items-center gap-3 rounded-xl border-2 p-5 transition-all",
                    theme === id
                      ? "border-primary bg-primary/5 shadow-sm"
                      : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
                  )}
                >
                  <Icon className="h-7 w-7" />
                  <span className="font-medium text-sm">{label}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 语言设置 */}
          <div className="space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.language")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.languageDesc")}</p>
            </div>
            <div className="grid grid-cols-2 gap-4">
              {supportedLanguages.map((lang) => (
                <button
                  key={lang.code}
                  type="button"
                  onClick={() => setLanguage(lang.code as LanguageCode)}
                  className={cn(
                    "flex items-center gap-3 rounded-xl border-2 p-4 transition-all",
                    language === lang.code
                      ? "border-primary bg-primary/5 shadow-sm"
                      : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
                  )}
                >
                  <Languages className="h-5 w-5" />
                  <span className="font-medium text-sm">{lang.name}</span>
                </button>
              ))}
            </div>
          </div>

          {/* 通知设置 */}
          <div className="space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.notifications")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.notificationsDesc")}</p>
            </div>
            <div className="flex items-center justify-between rounded-xl border bg-card p-5">
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

          {/* 关于 */}
          <div className="space-y-5">
            <div>
              <h3 className="mb-1 font-semibold text-lg">{t("settings.about")}</h3>
              <p className="text-muted-foreground text-sm">{t("settings.aboutDesc")}</p>
            </div>
            <div className="space-y-5 rounded-xl border bg-card p-5">
              <div className="flex items-center justify-between">
                <div className="space-y-1">
                  <p className="font-medium">{t("common.appName")}</p>
                  <p className="text-muted-foreground text-sm">
                    {t("settings.version")} {__APP_VERSION__}
                  </p>
                </div>
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
              </div>
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
