import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { IpType, StepProps } from "../types"

export function TargetStep({ state, onStateChange, onNext, onBack, domainName }: StepProps) {
  const { t } = useTranslation()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!state.target.trim()) return
    onNext()
  }

  const displayName = state.subdomain.trim() || "@"
  const fqdn = displayName === "@" ? domainName : `${displayName}.${domainName}`

  // 根据目的显示不同的提示
  const getPlaceholder = () => {
    switch (state.purpose) {
      case "website":
        return state.ipType === "ipv6" ? "2001:db8::1" : "192.168.1.1"
      case "subdomain":
        return "www.example.com"
      case "verification":
        return "v=spf1 include:..."
      default:
        return ""
    }
  }

  const getLabel = () => {
    switch (state.purpose) {
      case "website":
        return t("dns.wizard.target.ipLabel")
      case "subdomain":
        return t("dns.wizard.target.domainLabel")
      case "verification":
        return t("dns.wizard.target.txtLabel")
      default:
        return t("dns.wizard.target.label")
    }
  }

  const getHintKey = () => {
    switch (state.purpose) {
      case "website":
        return state.ipType === "ipv6" ? "dns.wizard.target.ipv6Hint" : "dns.wizard.target.ipv4Hint"
      case "subdomain":
        return "dns.wizard.target.cnameHint"
      case "verification":
        return "dns.wizard.target.txtHint"
      default:
        return ""
    }
  }

  // 获取预览提示
  const getPreviewHint = () => {
    const value = state.target.trim()
    if (!value) return null

    switch (state.purpose) {
      case "website":
        return t("dns.wizard.target.preview.website", { fqdn, ip: value })
      case "subdomain":
        return t("dns.wizard.target.preview.subdomain", { fqdn, target: value })
      case "verification":
        return t("dns.wizard.target.preview.verification", { fqdn, value })
      default:
        return null
    }
  }

  const previewHint = getPreviewHint()

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="text-center">
        <h3 className="font-medium text-lg">{t(`dns.wizard.target.title.${state.purpose}`)}</h3>
        <p className="mt-1 text-muted-foreground text-sm">
          {t(`dns.wizard.target.subtitle.${state.purpose}`, { fqdn })}
        </p>
      </div>

      {/* IP 类型切换（仅 website） */}
      {state.purpose === "website" && (
        <div className="flex justify-center gap-2">
          <Button
            type="button"
            variant={state.ipType === "ipv4" ? "default" : "outline"}
            size="sm"
            onClick={() => onStateChange({ ipType: "ipv4" as IpType, target: "" })}
          >
            IPv4
          </Button>
          <Button
            type="button"
            variant={state.ipType === "ipv6" ? "default" : "outline"}
            size="sm"
            onClick={() => onStateChange({ ipType: "ipv6" as IpType, target: "" })}
          >
            IPv6
          </Button>
        </div>
      )}

      <div className="space-y-2">
        <Label htmlFor="target">{getLabel()}</Label>
        <Input
          id="target"
          value={state.target}
          onChange={(e) => onStateChange({ target: e.target.value })}
          placeholder={getPlaceholder()}
          required
        />
        <p className="text-muted-foreground text-xs">{t(getHintKey())}</p>
      </div>

      {/* 预览提示 */}
      {previewHint && (
        <div className="rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950/30">
          <p className="text-blue-700 text-sm dark:text-blue-300">{previewHint}</p>
        </div>
      )}

      <div className="flex justify-between pt-2">
        <Button type="button" variant="outline" onClick={onBack}>
          {t("dns.wizard.back")}
        </Button>
        <Button type="submit" disabled={!state.target.trim()}>
          {t("dns.wizard.next")}
        </Button>
      </div>
    </form>
  )
}
