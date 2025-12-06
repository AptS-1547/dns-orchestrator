import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { ArrowLeft, Wrench } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DnsLookup } from "./DnsLookup"
import { TracerouteLookup } from "./TracerouteLookup"
import { WhoisLookup } from "./WhoisLookup"

interface ToolboxPageProps {
  onBack: () => void
}

export function ToolboxPage({ onBack }: ToolboxPageProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-1 flex-col min-h-0 overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 border-b bg-background px-4 py-3 sm:gap-4 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex items-center gap-2">
          <Wrench className="h-5 w-5 text-primary" />
          <h2 className="font-semibold text-xl">{t("toolbox.title")}</h2>
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1 min-h-0">
        <div className="mx-auto max-w-4xl space-y-4 p-4 sm:space-y-6 sm:p-6">
          <WhoisLookup />
          <DnsLookup />
          <TracerouteLookup />
        </div>
      </ScrollArea>
    </div>
  )
}
