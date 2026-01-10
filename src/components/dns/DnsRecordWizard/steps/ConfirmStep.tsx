import { Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { getMailPresetById, presetRecordToRecordData } from "../presets/mailPresets"
import type { StepProps, WizardState } from "../types"
import { getRecordTypeFromState } from "../types"

interface ConfirmStepProps extends StepProps {
  onSubmit: () => void
  isLoading: boolean
}

interface RecordPreview {
  type: string
  name: string
  value: string
  priority?: number
}

function getRecordPreviews(state: WizardState, domainName: string): RecordPreview[] {
  const displayName = state.subdomain.trim() || "@"
  const fqdn = displayName === "@" ? domainName : `${displayName}.${domainName}`

  // 邮件预设：返回多条记录
  if (state.purpose === "mail" && state.mailPresetId) {
    const preset = getMailPresetById(state.mailPresetId)
    if (preset) {
      return preset.records.map((record) => {
        const data = presetRecordToRecordData(record, domainName)
        if (data.type === "MX") {
          return {
            type: "MX",
            name: record.name === "@" ? domainName : `${record.name}.${domainName}`,
            value: data.content.exchange,
            priority: data.content.priority,
          }
        }
        return {
          type: "TXT",
          name: record.name === "@" ? domainName : `${record.name}.${domainName}`,
          value: (data.content as { text: string }).text,
        }
      })
    }
  }

  // 手动配置邮件（跳转高级模式）
  if (state.purpose === "mail" && !state.mailPresetId) {
    return []
  }

  // 其他类型：返回单条记录
  const recordType = getRecordTypeFromState(state)
  return [
    {
      type: recordType,
      name: fqdn,
      value: state.target,
    },
  ]
}

export function ConfirmStep({ state, onBack, onSubmit, isLoading, domainName }: ConfirmStepProps) {
  const { t } = useTranslation()

  const records = getRecordPreviews(state, domainName)
  const isManualMail = state.purpose === "mail" && !state.mailPresetId

  return (
    <div className="space-y-4">
      <div className="text-center">
        <h3 className="font-medium text-lg">{t("dns.wizard.confirm.title")}</h3>
        <p className="mt-1 text-muted-foreground text-sm">
          {isManualMail
            ? t("dns.wizard.confirm.manualMailSubtitle")
            : t("dns.wizard.confirm.subtitle", { count: records.length })}
        </p>
      </div>

      {/* 记录预览 */}
      {records.length > 0 && (
        <div className="space-y-2">
          {records.map((record, index) => (
            <div
              key={`${record.type}-${record.name}-${index}`}
              className="rounded-lg border bg-muted/30 p-3"
            >
              <div className="flex items-center gap-2">
                <span className="rounded bg-primary/10 px-2 py-0.5 font-medium text-primary text-xs">
                  {record.type}
                </span>
                <span className="font-mono text-sm">{record.name}</span>
              </div>
              <div className="mt-2 break-all font-mono text-muted-foreground text-sm">
                {record.priority !== undefined && (
                  <span className="mr-2 text-orange-600 dark:text-orange-400">
                    [{t("dns.priority")}: {record.priority}]
                  </span>
                )}
                {record.value}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* 提示 */}
      <div className="rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950/30">
        <p className="text-blue-700 text-sm dark:text-blue-300">
          {isManualMail ? t("dns.wizard.confirm.manualMailHint") : t("dns.wizard.confirm.hint")}
        </p>
      </div>

      <div className="flex justify-between pt-2">
        <Button type="button" variant="outline" onClick={onBack} disabled={isLoading}>
          {t("dns.wizard.back")}
        </Button>
        <Button onClick={onSubmit} disabled={isLoading}>
          {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
          {isManualMail
            ? t("dns.wizard.confirm.openAdvanced")
            : t("dns.wizard.confirm.create", { count: records.length })}
        </Button>
      </div>
    </div>
  )
}
