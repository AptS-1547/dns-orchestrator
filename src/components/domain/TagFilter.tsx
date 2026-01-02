import { Filter, X } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { useShallow } from "zustand/react/shallow"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { useDomainStore } from "@/stores"

/**
 * 标签筛选按钮组件（仅按钮部分）
 * 用于在搜索栏中与搜索框同行显示
 */
export function TagFilterButton() {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")

  const { selectedTags, setSelectedTags, getAllUsedTags } = useDomainStore(
    useShallow((state) => ({
      selectedTags: state.selectedTags,
      setSelectedTags: state.setSelectedTags,
      getAllUsedTags: state.getAllUsedTags,
    }))
  )

  const allTags = getAllUsedTags()

  // 过滤标签
  const filteredTags = useMemo(() => {
    if (!searchQuery.trim()) return allTags
    const query = searchQuery.toLowerCase()
    return allTags.filter((tag) => tag.toLowerCase().includes(query))
  }, [allTags, searchQuery])

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
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button variant="outline" size="sm" className="h-10 w-full sm:w-auto">
          <Filter className="mr-2 h-4 w-4" />
          {t("domain.tags.filter")}
          {selectedTags.size > 0 && (
            <Badge variant="secondary" className="ml-2 rounded-full px-1.5 py-0 text-xs">
              {selectedTags.size}
            </Badge>
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-56 p-0" align="start">
        <Command>
          <CommandInput
            placeholder={t("common.search")}
            value={searchQuery}
            onValueChange={setSearchQuery}
          />
          <CommandList className="max-h-[240px]">
            <CommandEmpty>{t("common.noMatch")}</CommandEmpty>
            <CommandGroup>
              <div className="px-2 py-1.5 text-muted-foreground text-xs">
                {t("domain.tags.filterLogicHint")}
              </div>
              {filteredTags.map((tag) => (
                <CommandItem key={tag} onSelect={() => handleToggleTag(tag)}>
                  <Checkbox checked={selectedTags.has(tag)} className="pointer-events-none mr-2" />
                  <span>{tag}</span>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  )
}

/**
 * 已选标签列表组件（独立显示）
 * 在有选中标签时显示，无标签时不占用空间
 */
export function SelectedTagsList() {
  const { t } = useTranslation()

  const { selectedTags, setSelectedTags, clearTagFilters } = useDomainStore(
    useShallow((state) => ({
      selectedTags: state.selectedTags,
      setSelectedTags: state.setSelectedTags,
      clearTagFilters: state.clearTagFilters,
    }))
  )

  const handleToggleTag = (tag: string) => {
    const newTags = new Set(selectedTags)
    if (newTags.has(tag)) {
      newTags.delete(tag)
    } else {
      newTags.add(tag)
    }
    setSelectedTags(Array.from(newTags))
  }

  if (selectedTags.size === 0) {
    return null
  }

  return (
    <TooltipProvider>
      <div className="flex flex-wrap items-center gap-2">
        <span className="shrink-0 text-muted-foreground text-xs">已选标签:</span>
        {Array.from(selectedTags).map((tag) => (
          <Tooltip key={tag}>
            <TooltipTrigger asChild>
              <button
                type="button"
                className="group inline-flex items-center gap-1 rounded-full bg-muted px-2 py-1 text-xs transition-colors hover:bg-muted/80"
                onClick={() => handleToggleTag(tag)}
              >
                <span className="max-w-32 truncate">{tag}</span>
                <span className="opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100">
                  <X className="h-3 w-3" />
                </span>
              </button>
            </TooltipTrigger>
            {tag.length > 12 && <TooltipContent>{tag}</TooltipContent>}
          </Tooltip>
        ))}
        <Button variant="ghost" size="sm" onClick={clearTagFilters} className="h-8 shrink-0">
          {t("common.clearAll")}
        </Button>
      </div>
    </TooltipProvider>
  )
}

/**
 * 标签筛选组件（完整版，包含按钮和已选标签）
 * 向后兼容的组件，内部使用 TagFilterButton 和 SelectedTagsList
 */
export function TagFilter() {
  const getAllUsedTags = useDomainStore((state) => state.getAllUsedTags)
  const allTags = getAllUsedTags()

  if (allTags.length === 0) {
    return null
  }

  return (
    <div className="space-y-2">
      <TagFilterButton />
      <SelectedTagsList />
    </div>
  )
}
