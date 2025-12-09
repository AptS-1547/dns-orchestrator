import { Settings, Wrench } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"

interface SidebarFooterProps {
  onOpenToolbox?: () => void
  onOpenSettings?: () => void
}

export function SidebarFooter({ onOpenToolbox, onOpenSettings }: SidebarFooterProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-1 border-t p-2">
      <Button variant="ghost" className="h-10 w-full justify-start gap-3" onClick={onOpenToolbox}>
        <Wrench className="h-4 w-4" />
        <span>{t("toolbox.title")}</span>
      </Button>
      <Button variant="ghost" className="h-10 w-full justify-start gap-3" onClick={onOpenSettings}>
        <Settings className="h-4 w-4" />
        <span>{t("settings.title")}</span>
      </Button>
    </div>
  )
}
