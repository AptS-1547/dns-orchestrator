import { Github } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Label } from "@/components/ui/label"
import { SettingItem, SettingRow, SettingSection } from "@/components/ui/setting-section"
import { Switch } from "@/components/ui/switch"
import { EXTERNAL_LINKS } from "@/constants"
import { ENV } from "@/lib/env"
import { openExternal } from "@/lib/open-external"
import { useSettingsStore } from "@/stores/settingsStore"
import { UpdateChecker } from "../UpdateChecker"

/**
 * 关于 Tab
 * 包含版本信息、更新检查功能和调试模式
 */
export function AboutTab() {
  const { t } = useTranslation()
  const { debugMode, setDebugMode } = useSettingsStore()

  return (
    <div className="space-y-6 sm:space-y-8">
      <SettingSection title={t("settings.about")} description={t("settings.aboutDesc")}>
        <SettingItem className="space-y-4 sm:space-y-5">
          {/* 版本信息 */}
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

          {/* 更新检查 */}
          <div className="border-t pt-2">
            <UpdateChecker />
          </div>
        </SettingItem>
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
    </div>
  )
}
