import { CheckCircle, Globe, Link, Mail } from "lucide-react"
import { useTranslation } from "react-i18next"
import { cn } from "@/lib/utils"
import type { WizardPurpose } from "../types"

interface PurposeStepProps {
  onSelect: (purpose: WizardPurpose) => void
}

interface PurposeOption {
  id: WizardPurpose
  icon: React.ReactNode
  labelKey: string
  descKey: string
}

const purposes: PurposeOption[] = [
  {
    id: "website",
    icon: <Globe className="h-6 w-6" />,
    labelKey: "dns.wizard.purpose.website",
    descKey: "dns.wizard.purpose.websiteDesc",
  },
  {
    id: "mail",
    icon: <Mail className="h-6 w-6" />,
    labelKey: "dns.wizard.purpose.mail",
    descKey: "dns.wizard.purpose.mailDesc",
  },
  {
    id: "subdomain",
    icon: <Link className="h-6 w-6" />,
    labelKey: "dns.wizard.purpose.subdomain",
    descKey: "dns.wizard.purpose.subdomainDesc",
  },
  {
    id: "verification",
    icon: <CheckCircle className="h-6 w-6" />,
    labelKey: "dns.wizard.purpose.verification",
    descKey: "dns.wizard.purpose.verificationDesc",
  },
]

export function PurposeStep({ onSelect }: PurposeStepProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-4">
      <div className="text-center">
        <h3 className="font-medium text-lg">{t("dns.wizard.purpose.title")}</h3>
        <p className="mt-1 text-muted-foreground text-sm">{t("dns.wizard.purpose.subtitle")}</p>
      </div>

      <div className="grid gap-3">
        {purposes.map((purpose) => (
          <button
            key={purpose.id}
            type="button"
            onClick={() => onSelect(purpose.id)}
            className={cn(
              "flex items-center gap-4 rounded-lg border p-4 text-left transition-colors",
              "hover:border-primary hover:bg-accent"
            )}
          >
            <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
              {purpose.icon}
            </div>
            <div className="min-w-0 flex-1">
              <div className="font-medium">{t(purpose.labelKey)}</div>
              <div className="text-muted-foreground text-sm">{t(purpose.descKey)}</div>
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}
