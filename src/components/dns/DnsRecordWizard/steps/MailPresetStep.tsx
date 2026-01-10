import { Settings } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { MAIL_PRESETS } from "../presets/mailPresets"
import type { StepProps } from "../types"

export function MailPresetStep({ state, onStateChange, onNext, onBack }: StepProps) {
  const { t } = useTranslation()

  const handleSelectPreset = (presetId: string | null) => {
    onStateChange({ mailPresetId: presetId })
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onNext()
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="text-center">
        <h3 className="text-lg font-medium">{t("dns.wizard.mail.title")}</h3>
        <p className="mt-1 text-muted-foreground text-sm">{t("dns.wizard.mail.subtitle")}</p>
      </div>

      <div className="grid gap-3">
        {MAIL_PRESETS.map((preset) => (
          <button
            key={preset.id}
            type="button"
            onClick={() => handleSelectPreset(preset.id)}
            className={cn(
              "flex items-center gap-4 rounded-lg border p-4 text-left transition-colors",
              state.mailPresetId === preset.id
                ? "border-primary bg-primary/5"
                : "hover:border-primary hover:bg-accent"
            )}
          >
            <div className="min-w-0 flex-1">
              <div className="font-medium">{t(preset.nameKey)}</div>
              {preset.descriptionKey && (
                <div className="text-muted-foreground text-sm">{t(preset.descriptionKey)}</div>
              )}
              <div className="mt-1 text-muted-foreground text-xs">
                {t("dns.wizard.mail.recordCount", { count: preset.records.length })}
              </div>
            </div>
            {state.mailPresetId === preset.id && (
              <div className="text-primary">
                <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
            )}
          </button>
        ))}

        {/* 手动配置选项 */}
        <button
          type="button"
          onClick={() => handleSelectPreset(null)}
          className={cn(
            "flex items-center gap-4 rounded-lg border p-4 text-left transition-colors",
            state.mailPresetId === null
              ? "border-primary bg-primary/5"
              : "hover:border-primary hover:bg-accent"
          )}
        >
          <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-muted">
            <Settings className="h-5 w-5 text-muted-foreground" />
          </div>
          <div className="min-w-0 flex-1">
            <div className="font-medium">{t("dns.wizard.mail.manual")}</div>
            <div className="text-muted-foreground text-sm">{t("dns.wizard.mail.manualDesc")}</div>
          </div>
          {state.mailPresetId === null && (
            <div className="text-primary">
              <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
          )}
        </button>
      </div>

      <div className="flex justify-between pt-2">
        <Button type="button" variant="outline" onClick={onBack}>
          {t("dns.wizard.back")}
        </Button>
        <Button type="submit">{t("dns.wizard.next")}</Button>
      </div>
    </form>
  )
}
