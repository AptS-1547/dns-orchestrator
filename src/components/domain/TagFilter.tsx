import { Filter, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useShallow } from "zustand/react/shallow"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { useDomainStore } from "@/stores"

export function TagFilter() {
  const { t } = useTranslation()

  const { selectedTags, setSelectedTags, clearTagFilters, getAllUsedTags } = useDomainStore(
    useShallow((state) => ({
      selectedTags: state.selectedTags,
      setSelectedTags: state.setSelectedTags,
      clearTagFilters: state.clearTagFilters,
      getAllUsedTags: state.getAllUsedTags,
    }))
  )

  const allTags = getAllUsedTags()

  const handleToggleTag = (tag: string) => {
    const newTags = new Set(selectedTags)
    if (newTags.has(tag)) {
      newTags.delete(tag)
    } else {
      newTags.add(tag)
    }
    setSelectedTags(Array.from(newTags))
  }

  if (allTags.length === 0) {
    return null
  }

  return (
    <TooltipProvider>
      <div className="flex items-center gap-2">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="h-8">
              <Filter className="mr-2 h-4 w-4" />
              {t("domain.tags.filter")}
              {selectedTags.size > 0 && (
                <Badge variant="secondary" className="ml-2 rounded-full px-1.5 py-0 text-xs">
                  {selectedTags.size}
                </Badge>
              )}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start" className="w-56">
            <DropdownMenuLabel>
              <div>
                {t("domain.tags.filterByTag")}
                <span className="text-muted-foreground font-normal text-xs block mt-0.5">
                  {t("domain.tags.filterLogicHint")}
                </span>
              </div>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            {allTags.map((tag) => (
              <DropdownMenuCheckboxItem
                key={tag}
                checked={selectedTags.has(tag)}
                onCheckedChange={() => handleToggleTag(tag)}
                onSelect={(e) => e.preventDefault()}
              >
                {tag}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {/* 已选标签显示 */}
        {selectedTags.size > 0 && (
          <>
            <div className="flex flex-wrap gap-1.5">
              {Array.from(selectedTags).map((tag) => (
                <Tooltip key={tag}>
                  <TooltipTrigger asChild>
                    <Badge
                      variant="default"
                      className="cursor-pointer ring-2 ring-primary/20 max-w-[120px] sm:max-w-[150px] transition-all hover:ring-primary/40"
                      onClick={() => handleToggleTag(tag)}
                    >
                      <span className="text-xs truncate inline-block align-bottom max-w-[90px] sm:max-w-[120px]">
                        {tag}
                      </span>
                      <X className="ml-1 h-3 w-3 flex-shrink-0" />
                    </Badge>
                  </TooltipTrigger>
                  {tag.length > 12 && <TooltipContent>{tag}</TooltipContent>}
                </Tooltip>
              ))}
            </div>
            <Button variant="ghost" size="sm" onClick={clearTagFilters} className="h-8">
              {t("common.clearAll")}
            </Button>
          </>
        )}
      </div>
    </TooltipProvider>
  )
}
