import { Globe } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores"
import type { Domain, DomainStatus } from "@/types"

interface DomainTreeItemProps {
  domain: Domain
  isSelected: boolean
  onSelect: () => void
}

const statusConfig: Record<
  DomainStatus,
  { labelKey: string; variant: "default" | "secondary" | "destructive" | "outline" }
> = {
  active: { labelKey: "domain.status.active", variant: "default" },
  paused: { labelKey: "domain.status.paused", variant: "secondary" },
  pending: { labelKey: "domain.status.pending", variant: "outline" },
  error: { labelKey: "domain.status.error", variant: "destructive" },
  unknown: { labelKey: "domain.status.unknown", variant: "outline" },
}

export function DomainTreeItem({ domain, isSelected, onSelect }: DomainTreeItemProps) {
  const { t } = useTranslation()
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)
  const config = statusConfig[domain.status] ?? statusConfig.active

  return (
    <button
      type="button"
      onClick={onSelect}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
        "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
        isSelected && "bg-sidebar-accent text-sidebar-accent-foreground"
      )}
    >
      {/* 颜色色块（固定占位以保持对齐） */}
      <div
        className="h-3 w-0.5 shrink-0 rounded-full"
        style={{
          backgroundColor: domain.metadata?.color
            ? getDomainColor(domain.metadata.color as DomainColorKey, isDark)
            : "transparent",
        }}
      />
      <Globe className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
      <span className="flex-1 truncate text-left">{domain.name}</span>
      <Badge variant={config.variant} className="px-1.5 py-0 text-[10px]">
        {t(config.labelKey)}
      </Badge>
    </button>
  )
}
