import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import type { StepProps } from "../types"

const COMMON_SUBDOMAINS = ["www", "@", "api", "blog", "mail", "shop"]

export function SubdomainStep({ state, onStateChange, onNext, onBack, domainName }: StepProps) {
  const { t } = useTranslation()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onNext()
  }

  const displayName = state.subdomain.trim() || "@"
  const fqdn = displayName === "@" ? domainName : `${displayName}.${domainName}`

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="text-center">
        <h3 className="font-medium text-lg">{t("dns.wizard.subdomain.title")}</h3>
        <p className="mt-1 text-muted-foreground text-sm">{t("dns.wizard.subdomain.subtitle")}</p>
      </div>

      <div className="space-y-2">
        <Label htmlFor="subdomain">{t("dns.wizard.subdomain.label")}</Label>
        <div className="flex items-center gap-2">
          <Input
            id="subdomain"
            value={state.subdomain}
            onChange={(e) => onStateChange({ subdomain: e.target.value })}
            placeholder="@"
            className="flex-1"
          />
          <span className="text-muted-foreground text-sm">.{domainName}</span>
        </div>
        <p className="text-muted-foreground text-xs">{t("dns.wizard.subdomain.hint")}</p>
      </div>

      {/* 常用子域名快捷选择 */}
      <div className="space-y-2">
        <Label className="text-muted-foreground text-xs">{t("dns.wizard.subdomain.common")}</Label>
        <div className="flex flex-wrap gap-2">
          {COMMON_SUBDOMAINS.map((sub) => (
            <Button
              key={sub}
              type="button"
              variant={
                state.subdomain === sub || (sub === "@" && !state.subdomain) ? "default" : "outline"
              }
              size="sm"
              onClick={() => onStateChange({ subdomain: sub === "@" ? "" : sub })}
            >
              {sub}
            </Button>
          ))}
        </div>
      </div>

      {/* 预览 */}
      <div className="rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950/30">
        <p className="text-blue-700 text-sm dark:text-blue-300">
          {t("dns.wizard.subdomain.preview", { fqdn })}
        </p>
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
