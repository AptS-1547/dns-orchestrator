import { Filter, X } from "lucide-react"
import { useTranslation } from "react-i18next"
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
import { useDomainStore } from "@/stores"
import { useShallow } from "zustand/react/shallow"

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
          <DropdownMenuLabel>{t("domain.tags.filterByTag")}</DropdownMenuLabel>
          <DropdownMenuSeparator />
          {allTags.map((tag) => (
            <DropdownMenuCheckboxItem
              key={tag}
              checked={selectedTags.has(tag)}
              onCheckedChange={() => handleToggleTag(tag)}
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
              <Badge
                key={tag}
                variant="default"
                className="cursor-pointer"
                onClick={() => handleToggleTag(tag)}
              >
                {tag}
                <X className="ml-1 h-3 w-3" />
              </Badge>
            ))}
          </div>
          <Button variant="ghost" size="sm" onClick={clearTagFilters} className="h-8">
            {t("common.clearAll")}
          </Button>
        </>
      )}
    </div>
  )
}
