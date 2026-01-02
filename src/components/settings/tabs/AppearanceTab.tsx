import { Languages, Monitor, Moon, Sun } from "lucide-react"
import { useTranslation } from "react-i18next"
import { SettingSection } from "@/components/ui/setting-section"
import { type LanguageCode, supportedLanguages } from "@/i18n"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores/settingsStore"

/**
 * 外观设置 Tab
 * 包含主题设置和语言设置
 */
export function AppearanceTab() {
  const { t } = useTranslation()
  const { theme, language, setTheme, setLanguage } = useSettingsStore()

  const themes = [
    { id: "light" as const, label: t("settings.themeLight"), icon: Sun },
    { id: "dark" as const, label: t("settings.themeDark"), icon: Moon },
    { id: "system" as const, label: t("settings.themeSystem"), icon: Monitor },
  ]

  return (
    <div className="space-y-6 sm:space-y-8">
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
    </div>
  )
}
