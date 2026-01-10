import { Globe, Plus } from "lucide-react"
import { memo } from "react"
import { useTranslation } from "react-i18next"
import { DomainFavoriteButton } from "@/components/domain/DomainFavoriteButton"
import { DomainMetadataEditor } from "@/components/domain/DomainMetadataEditor"
import { DomainTagList } from "@/components/domain/DomainTagList"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { statusConfig } from "./constants"
import type { DomainItemBaseProps } from "./types"

export const DomainItemMobile = memo(function DomainItemMobile({
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
  const hasTags = (domain.metadata?.tags?.length ?? 0) > 0
  const hasColor = domain.metadata?.color && domain.metadata.color !== "none"

  return (
    <Card
      className={cn(
        "relative overflow-hidden transition-all",
        isBatchMode && "cursor-pointer",
        isSelected && "ring-2 ring-primary"
      )}
      onClick={isBatchMode ? onClick : undefined}
    >
      {/* Color indicator bar (top edge) */}
      {hasColor && (
        <div
          className="absolute inset-x-0 top-0 h-1"
          style={{
            backgroundColor: getDomainColor(domain.metadata?.color as DomainColorKey, isDark),
          }}
        />
      )}

      <div className={cn("p-3", hasColor && "pt-4")}>
        {/* First row: checkbox/favorite + icon + name + status + edit */}
        <div className="flex items-center gap-2">
          {isBatchMode ? (
            <Checkbox
              checked={isSelected}
              onCheckedChange={() => onClick?.()}
              onClick={(e) => e.stopPropagation()}
            />
          ) : (
            <DomainFavoriteButton
              accountId={accountId}
              domainId={domain.id}
              isFavorite={domain.metadata?.isFavorite ?? false}
            />
          )}

          <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />

          <button
            type="button"
            className="min-w-0 flex-1 truncate text-left font-medium"
            onClick={!isBatchMode ? onClick : undefined}
          >
            {domain.name}
          </button>

          <Badge variant={config.variant} className="shrink-0">
            {t(config.labelKey)}
          </Badge>

          {!isBatchMode && (
            <DomainMetadataEditor
              accountId={accountId}
              domainId={domain.id}
              currentMetadata={domain.metadata}
            >
              <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                <Plus className="h-4 w-4" />
              </Button>
            </DomainMetadataEditor>
          )}
        </div>

        {/* Second row: tags (only if has tags) */}
        {hasTags && (
          <div className="mt-2 border-t pt-2">
            <DomainTagList
              tags={domain.metadata?.tags ?? []}
              onClickTag={isBatchMode ? undefined : onClickTag}
              className="flex-wrap"
            />
          </div>
        )}
      </div>
    </Card>
  )
})
