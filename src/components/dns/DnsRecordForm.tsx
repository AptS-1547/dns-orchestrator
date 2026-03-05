import { Loader2 } from "lucide-react"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Switch } from "@/components/ui/switch"
import { DNS } from "@/constants"
import { useDnsStore, useDomainStore } from "@/stores"
import { useSettingsStore } from "@/stores/settingsStore"
import type { DnsRecord, DnsRecordType, RecordData } from "@/types"
import { RECORD_TYPE_INFO, RECORD_TYPES, TTL_OPTIONS } from "@/types/dns"
import { RECORD_TYPE_CONFIGS, type RecordFieldConfig } from "./record-type-configs"

interface DnsRecordFormProps {
  accountId: string
  domainId: string
  record?: DnsRecord | null
  onClose: () => void
  supportsProxy?: boolean
}

type FieldValues = Record<string, string | number>

export function DnsRecordForm({
  accountId,
  domainId,
  record,
  onClose,
  supportsProxy = false,
}: DnsRecordFormProps) {
  const { t } = useTranslation()
  const { createRecord, updateRecord, isLoading } = useDnsStore()
  const isEditing = !!record

  const [formType, setFormType] = useState<DnsRecordType>(record?.data.type ?? "A")
  const [name, setName] = useState(record?.name ?? "")
  const [ttl, setTtl] = useState(record?.ttl ?? DNS.DEFAULT_TTL)
  const [proxied, setProxied] = useState(record?.proxied)
  const [values, setValues] = useState<FieldValues>(() =>
    record ? { ...(record.data.content as FieldValues) } : { ...RECORD_TYPE_CONFIGS.A.defaults }
  )

  const config = RECORD_TYPE_CONFIGS[formType]
  const typeInfo = RECORD_TYPE_INFO[formType]

  // 获取当前域名
  const currentDomain = useDomainStore((state) => {
    const domains = state.getDomainsForAccount(accountId)
    return domains.find((d) => d.id === domainId) ?? null
  })

  const showRecordHints = useSettingsStore((state) => state.showRecordHints)

  const getFQDN = useCallback(
    (inputName: string): string => {
      if (!currentDomain?.name) return ""
      const cleanName = inputName.trim()
      if (!cleanName || cleanName === "@") return currentDomain.name
      return `${cleanName}.${currentDomain.name}`
    },
    [currentDomain?.name]
  )

  const updateValue = (key: string, val: string | number) => {
    setValues((prev) => ({ ...prev, [key]: val }))
  }

  const getCurrentValue = (): string => String(values[config.primaryValueKey] ?? "")

  const getRecordHint = (): string | null => {
    const fqdn = getFQDN(name)
    const value = getCurrentValue()
    if (!(value && fqdn)) return null

    const params: Record<string, string | number> = { fqdn, value, ...values }
    let hint = t(`dns.recordHints.${formType}`, params)

    if (supportsProxy && proxied && (formType === "A" || formType === "AAAA")) {
      hint += ` ${t("dns.recordHints.proxyEnabled")}`
    }

    return hint
  }

  const recordHint = getRecordHint()

  const buildRecordData = (): RecordData => {
    return { type: formType, content: { ...values } } as RecordData
  }

  const buildRequest = () => ({
    domainId,
    name: name || "@",
    ttl,
    data: buildRecordData(),
    proxied: supportsProxy ? proxied : undefined,
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (isEditing && record) {
      const success = await updateRecord(accountId, record.id, buildRequest())
      if (success) onClose()
    } else {
      const result = await createRecord(accountId, buildRequest())
      if (result) onClose()
    }
  }

  const handleTypeChange = (newType: DnsRecordType) => {
    setFormType(newType)
    setValues({ ...RECORD_TYPE_CONFIGS[newType].defaults })
  }

  const renderRecordHint = () => {
    if (showRecordHints && recordHint) {
      return (
        <div className="fade-in animate-in rounded-md border border-blue-200 bg-blue-50 p-3 duration-200 dark:border-blue-800 dark:bg-blue-950/30">
          <p className="text-blue-700 text-sm leading-relaxed dark:text-blue-300">{recordHint}</p>
        </div>
      )
    }
    return (
      <p className="text-muted-foreground text-xs">
        {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
      </p>
    )
  }

  const renderField = (field: RecordFieldConfig) => {
    if (field.type === "select") {
      return (
        <Select
          value={String(values[field.key] ?? "")}
          onValueChange={(v) => updateValue(field.key, v)}
        >
          <SelectTrigger>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {field.selectOptions?.map((opt) => (
              <SelectItem key={opt.value} value={opt.value}>
                {opt.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      )
    }

    return (
      <Input
        id={field.key}
        type={field.type === "number" ? "number" : undefined}
        value={values[field.key] ?? ""}
        onChange={(e) =>
          updateValue(
            field.key,
            field.type === "number" ? Number.parseInt(e.target.value, 10) : e.target.value
          )
        }
        placeholder={field.placeholder}
        required={field.required}
        min={field.min}
        max={field.max}
      />
    )
  }

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{isEditing ? t("dns.editRecord") : t("dns.addRecord")}</DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Type */}
          <div className="space-y-2">
            <Label htmlFor="type">{t("common.type")}</Label>
            <Select
              value={formType}
              onValueChange={(v) => handleTypeChange(v as DnsRecordType)}
              disabled={isEditing}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {RECORD_TYPES.map((type) => (
                  <SelectItem key={type} value={type}>
                    <span className="font-medium">{type}</span>
                    <span className="ml-2 text-muted-foreground text-xs">
                      - {t(RECORD_TYPE_INFO[type].descriptionKey)}
                    </span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Name */}
          <div className="space-y-2">
            <Label htmlFor="name">{t("dns.name")}</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("dns.namePlaceholder")}
            />
            <p className="text-muted-foreground text-xs">{t("dns.nameHelp")}</p>
          </div>

          {/* 配置驱动的类型字段 */}
          {config.fields.map((field) => (
            <div key={field.key} className="space-y-2">
              <Label htmlFor={field.key}>{t(field.labelKey)}</Label>
              {renderField(field)}
              {field.key === config.primaryValueKey && renderRecordHint()}
            </div>
          ))}

          {/* TTL */}
          <div className="space-y-2">
            <Label htmlFor="ttl">{t("dns.ttl")}</Label>
            <Select value={String(ttl)} onValueChange={(v) => setTtl(Number.parseInt(v, 10))}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {TTL_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={String(option.value)}>
                    {t(option.labelKey, { count: "count" in option ? option.count : undefined })}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Proxied */}
          {supportsProxy && (
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="proxied">{t("dns.proxy")}</Label>
                <p className="text-muted-foreground text-xs">{t("dns.proxyHelp")}</p>
              </div>
              <Switch
                id="proxied"
                checked={proxied}
                onCheckedChange={(checked) => setProxied(checked)}
              />
            </div>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose} disabled={isLoading}>
              {t("common.cancel")}
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {isEditing ? t("common.save") : t("common.add")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
