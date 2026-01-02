import { Check } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Label } from "@/components/ui/label"
import { SettingItem, SettingRow, SettingSection } from "@/components/ui/setting-section"
import { Switch } from "@/components/ui/switch"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores/settingsStore"

/**
 * 功能设置 Tab
 * 包含通知、DNS 提示、分页模式设置
 */
export function FeaturesTab() {
  const { t } = useTranslation()
  const { paginationMode, showRecordHints, setPaginationMode, setShowRecordHints } =
    useSettingsStore()

  // 分页模式选项配置
  const paginationOptions = [
    {
      id: "infinite" as const,
      label: t("settings.infiniteScroll"),
      description: t("settings.infiniteScrollDesc"),
    },
    {
      id: "paginated" as const,
      label: t("settings.traditionalPagination"),
      description: t("settings.traditionalPaginationDesc"),
    },
  ]

  return (
    <div className="space-y-6 sm:space-y-8">
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
      <SettingSection title={t("settings.recordHints")} description={t("settings.recordHintsDesc")}>
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

      {/* 分页模式设置 - 改用 RadioGroup 风格 */}
      <SettingSection title={t("settings.pagination")} description={t("settings.paginationDesc")}>
        <div className="space-y-3">
          {paginationOptions.map((option) => (
            <label
              key={option.id}
              htmlFor={`pagination-${option.id}`}
              className={cn(
                "flex w-full cursor-pointer items-center justify-between rounded-xl border-2 p-4 transition-all sm:p-5",
                paginationMode === option.id
                  ? "border-primary bg-primary/5 shadow-sm"
                  : "border-border bg-card hover:border-accent-foreground/20 hover:bg-accent"
              )}
            >
              <input
                type="radio"
                id={`pagination-${option.id}`}
                name="paginationMode"
                value={option.id}
                checked={paginationMode === option.id}
                onChange={() => setPaginationMode(option.id)}
                className="sr-only"
              />
              <div className="text-left">
                <p className="font-medium text-sm">{option.label}</p>
                <p className="text-muted-foreground text-xs">{option.description}</p>
              </div>
              {paginationMode === option.id && <Check className="h-5 w-5 text-primary" />}
            </label>
          ))}
        </div>
      </SettingSection>
    </div>
  )
}
