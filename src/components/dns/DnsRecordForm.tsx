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
import { useDnsStore } from "@/stores"
import type {
  CreateDnsRecordRequest,
  DnsRecord,
  DnsRecordType,
  UpdateDnsRecordRequest,
} from "@/types"
import { RECORD_TYPE_INFO, TTL_OPTIONS } from "@/types/dns"
import { Loader2 } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"

interface DnsRecordFormProps {
  accountId: string
  domainId: string
  record?: DnsRecord | null
  onClose: () => void
  supportsProxy?: boolean
}

const RECORD_TYPES: DnsRecordType[] = ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SRV", "CAA"]

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

  const [formData, setFormData] = useState({
    type: (record?.type || "A") as DnsRecordType,
    name: record?.name || "",
    value: record?.value || "",
    ttl: record?.ttl || 300,
    priority: record?.priority || undefined,
    proxied: record?.proxied,
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    if (isEditing && record) {
      const updateRequest: UpdateDnsRecordRequest = {
        domainId,
        type: formData.type,
        name: formData.name || "@",
        value: formData.value,
        ttl: formData.ttl,
        priority: formData.priority,
        proxied: supportsProxy ? formData.proxied : undefined,
      }
      const success = await updateRecord(accountId, record.id, updateRequest)
      if (success) onClose()
    } else {
      const request: CreateDnsRecordRequest = {
        domainId,
        type: formData.type,
        name: formData.name || "@",
        value: formData.value,
        ttl: formData.ttl,
        priority: formData.priority,
        proxied: supportsProxy ? formData.proxied : undefined,
      }
      const result = await createRecord(accountId, request)
      if (result) onClose()
    }
  }

  const needsPriority = formData.type === "MX" || formData.type === "SRV"
  const typeInfo = RECORD_TYPE_INFO[formData.type]

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
              value={formData.type}
              onValueChange={(v) => setFormData({ ...formData, type: v as DnsRecordType })}
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
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder={t("dns.namePlaceholder")}
            />
            <p className="text-muted-foreground text-xs">{t("dns.nameHelp")}</p>
          </div>

          {/* Value */}
          <div className="space-y-2">
            <Label htmlFor="value">{t("dns.value")}</Label>
            <Input
              id="value"
              value={formData.value}
              onChange={(e) => setFormData({ ...formData, value: e.target.value })}
              placeholder={typeInfo?.example || t("dns.value")}
              required
            />
            <p className="text-muted-foreground text-xs">
              {typeInfo && t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo?.example}
            </p>
          </div>

          {/* Priority (for MX/SRV) */}
          {needsPriority && (
            <div className="space-y-2">
              <Label htmlFor="priority">{t("dns.priority")}</Label>
              <Input
                id="priority"
                type="number"
                value={formData.priority || ""}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    priority: e.target.value ? Number.parseInt(e.target.value) : undefined,
                  })
                }
                placeholder="10"
                min={0}
                max={65535}
              />
            </div>
          )}

          {/* TTL */}
          <div className="space-y-2">
            <Label htmlFor="ttl">{t("dns.ttl")}</Label>
            <Select
              value={String(formData.ttl)}
              onValueChange={(v) => setFormData({ ...formData, ttl: Number.parseInt(v) })}
            >
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

          {/* Proxied (仅 Cloudflare 等支持) */}
          {supportsProxy && (
            <div className="flex items-center justify-between">
              <div className="space-y-0.5">
                <Label htmlFor="proxied">{t("dns.proxy")}</Label>
                <p className="text-muted-foreground text-xs">{t("dns.proxyHelp")}</p>
              </div>
              <Switch
                id="proxied"
                checked={formData.proxied}
                onCheckedChange={(checked) => setFormData({ ...formData, proxied: checked })}
              />
            </div>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={onClose}>
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
