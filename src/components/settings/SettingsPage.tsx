import {
  Check,
  Download,
  Github,
  Languages,
  Monitor,
  Moon,
  RefreshCw,
  Settings,
  Sun,
  X,
} from "lucide-react"
import { useEffect } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { PageContainer } from "@/components/ui/page-container"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { SettingItem, SettingRow, SettingSection } from "@/components/ui/setting-section"
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
  const {
    theme,
    language,
    debugMode,
    paginationMode,
    showRecordHints,
    setTheme,
    setLanguage,
    setDebugMode,
    setPaginationMode,
    setShowRecordHints,
  } = useSettingsStore()
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

  const themes = [
    { id: "light" as const, label: t("settings.themeLight"), icon: Sun },
    { id: "dark" as const, label: t("settings.themeDark"), icon: Moon },
    { id: "system" as const, label: t("settings.themeSystem"), icon: Monitor },
  ]

  return (
    <PageLayout>
      <PageHeader title={t("settings.title")} icon={<Settings className="h-5 w-5" />} />

      <PageContainer maxWidth="max-w-3xl" className="space-y-6 sm:space-y-8">
        {/* 主题设置 */}
        <SettingSection title={t("settings.appearance")} description={t("settings.theme")}>
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
        </SettingSection>

        {/* 语言设置 */}
        <SettingSection title={t("settings.language")} description={t("settings.languageDesc")}>
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
        </SettingSection>

        {/* 通知设置 */}
        <SettingSection
          title={t("settings.notifications")}
          description={t("settings.notificationsDesc")}
        >
          <SettingItem>
            <SettingRow
              label={
                <Label htmlFor="notifications" className="font-medium text-sm">
                  {t("settings.operationNotifications")}
                </Label>
              }
              description={t("settings.operationNotificationsDesc")}
              control={<Switch id="notifications" defaultChecked />}
            />
          </SettingItem>
        </SettingSection>

        {/* DNS 记录提示设置 */}
        <SettingSection
          title={t("settings.recordHints")}
          description={t("settings.recordHintsDesc")}
        >
          <SettingItem>
            <SettingRow
              label={
                <Label htmlFor="record-hints" className="font-medium text-sm">
                  {t("settings.showRecordHints")}
                </Label>
              }
              description={t("settings.showRecordHintsDesc")}
              control={
                <Switch
                  id="record-hints"
                  checked={showRecordHints}
                  onCheckedChange={setShowRecordHints}
                />
              }
            />
          </SettingItem>
        </SettingSection>

        {/* 分页模式设置 */}
        <SettingSection title={t("settings.pagination")} description={t("settings.paginationDesc")}>
          <div className="space-y-3">
            <button
              type="button"
              onClick={() => setPaginationMode("infinite")}
              className={cn(
                "flex w-full items-center justify-between rounded-xl border-2 p-4 transition-all sm:p-5",
                paginationMode === "infinite"
                  ? "border-primary bg-primary/5 shadow-sm"
                  : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
              )}
            >
              <div className="text-left">
                <p className="font-medium text-sm">{t("settings.infiniteScroll")}</p>
                <p className="text-muted-foreground text-xs">{t("settings.infiniteScrollDesc")}</p>
              </div>
              {paginationMode === "infinite" && <Check className="h-5 w-5 text-primary" />}
            </button>

            <button
              type="button"
              onClick={() => setPaginationMode("paginated")}
              className={cn(
                "flex w-full items-center justify-between rounded-xl border-2 p-4 transition-all sm:p-5",
                paginationMode === "paginated"
                  ? "border-primary bg-primary/5 shadow-sm"
                  : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
              )}
            >
              <div className="text-left">
                <p className="font-medium text-sm">{t("settings.traditionalPagination")}</p>
                <p className="text-muted-foreground text-xs">
                  {t("settings.traditionalPaginationDesc")}
                </p>
              </div>
              {paginationMode === "paginated" && <Check className="h-5 w-5 text-primary" />}
            </button>
          </div>
        </SettingSection>

        {/* 调试模式设置 - 仅开发环境显示 */}
        {ENV.isDev && (
          <SettingSection title={t("settings.debug")} description={t("settings.debugDesc")}>
            <SettingItem>
              <SettingRow
                label={
                  <Label htmlFor="debug-mode" className="font-medium text-sm">
                    {t("settings.debugMode")}
                  </Label>
                }
                description={t("settings.debugModeDesc")}
                control={
                  <Switch id="debug-mode" checked={debugMode} onCheckedChange={setDebugMode} />
                }
              />
            </SettingItem>
          </SettingSection>
        )}

        {/* 关于 */}
        <SettingSection title={t("settings.about")} description={t("settings.aboutDesc")}>
          <SettingItem className="space-y-4 sm:space-y-5">
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
                    <pre className="whitespace-pre-wrap break-all text-xs">
                      {getUpdateNotes(available)}
                    </pre>
                  </ScrollArea>
                </div>
              )}
            </div>
          </SettingItem>
        </SettingSection>
      </PageContainer>
    </PageLayout>
  )
}
