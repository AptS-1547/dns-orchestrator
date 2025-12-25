import { Loader2 } from "lucide-react"
import { useState } from "react"
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
import { useDnsStore } from "@/stores"
import type { DnsRecord, DnsRecordType, RecordData } from "@/types"
import { RECORD_TYPE_INFO, RECORD_TYPES, TTL_OPTIONS } from "@/types/dns"

interface DnsRecordFormProps {
  accountId: string
  domainId: string
  record?: DnsRecord | null
  onClose: () => void
  supportsProxy?: boolean
}

// 表单数据类型，根据记录类型包含不同字段
type FormData = {
  type: DnsRecordType
  name: string
  ttl: number
  proxied?: boolean
} & (
  | { type: "A"; address: string }
  | { type: "AAAA"; address: string }
  | { type: "CNAME"; target: string }
  | { type: "MX"; priority: number; exchange: string }
  | { type: "TXT"; text: string }
  | { type: "NS"; nameserver: string }
  | { type: "SRV"; priority: number; weight: number; port: number; target: string }
  | { type: "CAA"; flags: number; tag: string; value: string }
)

// 从 DnsRecord 初始化表单数据
function initFormData(record: DnsRecord | null | undefined): FormData {
  const baseData = {
    name: record?.name || "",
    ttl: record?.ttl || DNS.DEFAULT_TTL,
    proxied: record?.proxied,
  }

  if (!record) {
    return { ...baseData, type: "A", address: "" }
  }

  const { data } = record

  switch (data.type) {
    case "A":
    case "AAAA":
      return { ...baseData, type: data.type, address: data.content.address }
    case "CNAME":
      return { ...baseData, type: "CNAME", target: data.content.target }
    case "MX":
      return {
        ...baseData,
        type: "MX",
        priority: data.content.priority,
        exchange: data.content.exchange,
      }
    case "TXT":
      return { ...baseData, type: "TXT", text: data.content.text }
    case "NS":
      return { ...baseData, type: "NS", nameserver: data.content.nameserver }
    case "SRV":
      return {
        ...baseData,
        type: "SRV",
        priority: data.content.priority,
        weight: data.content.weight,
        port: data.content.port,
        target: data.content.target,
      }
    case "CAA":
      return {
        ...baseData,
        type: "CAA",
        flags: data.content.flags,
        tag: data.content.tag,
        value: data.content.value,
      }
  }
}

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

  const [formData, setFormData] = useState<FormData>(initFormData(record))

  // 构建 RecordData
  const buildRecordData = (): RecordData => {
    switch (formData.type) {
      case "A":
      case "AAAA":
        return { type: formData.type, content: { address: formData.address } }
      case "CNAME":
        return { type: "CNAME", content: { target: formData.target } }
      case "MX":
        return {
          type: "MX",
          content: { priority: formData.priority, exchange: formData.exchange },
        }
      case "TXT":
        return { type: "TXT", content: { text: formData.text } }
      case "NS":
        return { type: "NS", content: { nameserver: formData.nameserver } }
      case "SRV":
        return {
          type: "SRV",
          content: {
            priority: formData.priority,
            weight: formData.weight,
            port: formData.port,
            target: formData.target,
          },
        }
      case "CAA":
        return {
          type: "CAA",
          content: { flags: formData.flags, tag: formData.tag, value: formData.value },
        }
    }
  }

  // 构建请求对象
  const buildRequest = () => ({
    domainId,
    name: formData.name || "@",
    ttl: formData.ttl,
    data: buildRecordData(),
    proxied: supportsProxy ? formData.proxied : undefined,
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

  // 处理类型切换，重置表单数据
  const handleTypeChange = (newType: DnsRecordType) => {
    const baseData = { name: formData.name, ttl: formData.ttl, proxied: formData.proxied }

    switch (newType) {
      case "A":
      case "AAAA":
        setFormData({ ...baseData, type: newType, address: "" })
        break
      case "CNAME":
        setFormData({ ...baseData, type: "CNAME", target: "" })
        break
      case "MX":
        setFormData({ ...baseData, type: "MX", priority: 10, exchange: "" })
        break
      case "TXT":
        setFormData({ ...baseData, type: "TXT", text: "" })
        break
      case "NS":
        setFormData({ ...baseData, type: "NS", nameserver: "" })
        break
      case "SRV":
        setFormData({ ...baseData, type: "SRV", priority: 10, weight: 5, port: 80, target: "" })
        break
      case "CAA":
        setFormData({ ...baseData, type: "CAA", flags: 0, tag: "issue", value: "" })
        break
    }
  }

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
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder={t("dns.namePlaceholder")}
            />
            <p className="text-muted-foreground text-xs">{t("dns.nameHelp")}</p>
          </div>

          {/* 根据类型渲染不同的字段 */}
          {(formData.type === "A" || formData.type === "AAAA") && (
            <div className="space-y-2">
              <Label htmlFor="address">{t("dns.value")}</Label>
              <Input
                id="address"
                value={formData.address}
                onChange={(e) => setFormData({ ...formData, address: e.target.value })}
                placeholder={typeInfo.example}
                required
              />
              <p className="text-muted-foreground text-xs">
                {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
              </p>
            </div>
          )}

          {formData.type === "CNAME" && (
            <div className="space-y-2">
              <Label htmlFor="target">{t("dns.value")}</Label>
              <Input
                id="target"
                value={formData.target}
                onChange={(e) => setFormData({ ...formData, target: e.target.value })}
                placeholder={typeInfo.example}
                required
              />
              <p className="text-muted-foreground text-xs">
                {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
              </p>
            </div>
          )}

          {formData.type === "MX" && (
            <>
              <div className="space-y-2">
                <Label htmlFor="priority">{t("dns.priority")}</Label>
                <Input
                  id="priority"
                  type="number"
                  value={formData.priority}
                  onChange={(e) =>
                    setFormData({ ...formData, priority: Number.parseInt(e.target.value, 10) })
                  }
                  placeholder="10"
                  min={0}
                  max={65535}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="exchange">{t("dns.value")}</Label>
                <Input
                  id="exchange"
                  value={formData.exchange}
                  onChange={(e) => setFormData({ ...formData, exchange: e.target.value })}
                  placeholder={typeInfo.example}
                  required
                />
                <p className="text-muted-foreground text-xs">
                  {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
                </p>
              </div>
            </>
          )}

          {formData.type === "TXT" && (
            <div className="space-y-2">
              <Label htmlFor="text">{t("dns.value")}</Label>
              <Input
                id="text"
                value={formData.text}
                onChange={(e) => setFormData({ ...formData, text: e.target.value })}
                placeholder={typeInfo.example}
                required
              />
              <p className="text-muted-foreground text-xs">
                {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
              </p>
            </div>
          )}

          {formData.type === "NS" && (
            <div className="space-y-2">
              <Label htmlFor="nameserver">{t("dns.value")}</Label>
              <Input
                id="nameserver"
                value={formData.nameserver}
                onChange={(e) => setFormData({ ...formData, nameserver: e.target.value })}
                placeholder={typeInfo.example}
                required
              />
              <p className="text-muted-foreground text-xs">
                {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
              </p>
            </div>
          )}

          {formData.type === "SRV" && (
            <>
              <div className="space-y-2">
                <Label htmlFor="priority">{t("dns.priority")}</Label>
                <Input
                  id="priority"
                  type="number"
                  value={formData.priority}
                  onChange={(e) =>
                    setFormData({ ...formData, priority: Number.parseInt(e.target.value, 10) })
                  }
                  placeholder="10"
                  min={0}
                  max={65535}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="weight">{t("dns.weight")}</Label>
                <Input
                  id="weight"
                  type="number"
                  value={formData.weight}
                  onChange={(e) =>
                    setFormData({ ...formData, weight: Number.parseInt(e.target.value, 10) })
                  }
                  placeholder="5"
                  min={0}
                  max={65535}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="port">{t("dns.port")}</Label>
                <Input
                  id="port"
                  type="number"
                  value={formData.port}
                  onChange={(e) =>
                    setFormData({ ...formData, port: Number.parseInt(e.target.value, 10) })
                  }
                  placeholder="80"
                  min={0}
                  max={65535}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="target">{t("dns.target")}</Label>
                <Input
                  id="target"
                  value={formData.target}
                  onChange={(e) => setFormData({ ...formData, target: e.target.value })}
                  placeholder={typeInfo.example}
                  required
                />
                <p className="text-muted-foreground text-xs">
                  {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
                </p>
              </div>
            </>
          )}

          {formData.type === "CAA" && (
            <>
              <div className="space-y-2">
                <Label htmlFor="flags">{t("dns.flags")}</Label>
                <Input
                  id="flags"
                  type="number"
                  value={formData.flags}
                  onChange={(e) =>
                    setFormData({ ...formData, flags: Number.parseInt(e.target.value, 10) })
                  }
                  placeholder="0"
                  min={0}
                  max={255}
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="tag">{t("dns.tag")}</Label>
                <Select
                  value={formData.tag}
                  onValueChange={(v) => setFormData({ ...formData, tag: v })}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="issue">issue</SelectItem>
                    <SelectItem value="issuewild">issuewild</SelectItem>
                    <SelectItem value="iodef">iodef</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label htmlFor="value">{t("dns.value")}</Label>
                <Input
                  id="value"
                  value={formData.value}
                  onChange={(e) => setFormData({ ...formData, value: e.target.value })}
                  placeholder={typeInfo.example}
                  required
                />
                <p className="text-muted-foreground text-xs">
                  {t(typeInfo.descriptionKey)} - {t("common.example")}: {typeInfo.example}
                </p>
              </div>
            </>
          )}

          {/* TTL */}
          <div className="space-y-2">
            <Label htmlFor="ttl">{t("dns.ttl")}</Label>
            <Select
              value={String(formData.ttl)}
              onValueChange={(v) => setFormData({ ...formData, ttl: Number.parseInt(v, 10) })}
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
