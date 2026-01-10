import { Globe, Plus } from "lucide-react"
import { memo } from "react"
import { useTranslation } from "react-i18next"
import { DomainFavoriteButton } from "@/components/domain/DomainFavoriteButton"
import { DomainMetadataEditor } from "@/components/domain/DomainMetadataEditor"
import { DomainTagList } from "@/components/domain/DomainTagList"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { statusConfig } from "./constants"
import type { DomainItemBaseProps } from "./types"

export const DomainItemDesktop = memo(function DomainItemDesktop({
  domain,
  accountId,
  isBatchMode,
  isSelected,
  isDark,
  onClick,
  onClickTag,
}: DomainItemBaseProps) {
  const { t } = useTranslation()
  const config = statusConfig[domain.status] ?? statusConfig.active

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-3 py-2 text-left transition-colors",
        "hover:bg-accent hover:text-accent-foreground",
        isBatchMode && isSelected && "bg-accent ring-2 ring-primary"
      )}
    >
      {/* Batch mode checkbox */}
      {isBatchMode && <Checkbox checked={isSelected} className="pointer-events-none" />}

      {/* Favorite button (hidden in batch mode) */}
      {!isBatchMode && (
        <DomainFavoriteButton
          accountId={accountId}
          domainId={domain.id}
          isFavorite={domain.metadata?.isFavorite ?? false}
        />
      )}

      {/* Color indicator */}
      <div
        className="h-4 w-1 shrink-0 rounded-full"
        style={{
          backgroundColor: domain.metadata?.color
            ? getDomainColor(domain.metadata.color as DomainColorKey, isDark)
            : "transparent",
        }}
        aria-label={domain.metadata?.color ? `Color: ${domain.metadata.color}` : undefined}
      />

      {/* Domain icon */}
      <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />

      {/* Domain name */}
      <span className="min-w-[100px] shrink-0 truncate font-medium sm:min-w-[140px]">
        {domain.name}
      </span>

      {/* Tags (desktop: limited display) */}
      <div className="min-w-0 flex-1">
        <DomainTagList
          tags={domain.metadata?.tags ?? []}
          onClickTag={onClickTag}
          maxDisplay={3}
          className="hidden sm:flex"
        />
      </div>

      {/* Status badge */}
      <Badge variant={config.variant} className="shrink-0">
        {t(config.labelKey)}
      </Badge>

      {/* Edit button (hidden in batch mode) */}
      {!isBatchMode && (
        <DomainMetadataEditor
          accountId={accountId}
          domainId={domain.id}
          currentMetadata={domain.metadata}
        >
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 shrink-0"
            onClick={(e) => e.stopPropagation()}
          >
            <Plus className="h-4 w-4" />
          </Button>
        </DomainMetadataEditor>
      )}
    </button>
  )
})
