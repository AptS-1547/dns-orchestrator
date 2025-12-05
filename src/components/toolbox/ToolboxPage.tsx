import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { ArrowLeft, Wrench } from "lucide-react"
import { useTranslation } from "react-i18next"
import { DnsLookup } from "./DnsLookup"
import { WhoisLookup } from "./WhoisLookup"

interface ToolboxPageProps {
  onBack: () => void
}

export function ToolboxPage({ onBack }: ToolboxPageProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-4 border-b bg-background px-6 py-4">
        <Button variant="ghost" size="icon" onClick={onBack}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex items-center gap-2">
          <Wrench className="h-5 w-5 text-primary" />
          <h2 className="font-semibold text-xl">{t("toolbox.title")}</h2>
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="mx-auto max-w-4xl space-y-6 p-6">
          <WhoisLookup />
          <DnsLookup />
        </div>
      </ScrollArea>
    </div>
  )
}
