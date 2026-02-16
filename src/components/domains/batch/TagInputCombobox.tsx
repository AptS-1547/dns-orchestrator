import { ChevronDown, Plus } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import { Input } from "@/components/ui/input"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { cn } from "@/lib/utils"

interface TagInputComboboxProps {
  value: string
  onChange: (value: string) => void
  onAddTag: () => void
  onSelectTag: (tag: string) => void
  currentTags: string[]
  allTags: string[]
  placeholder?: string
  maxLength?: number
  inputId?: string
}

export function TagInputCombobox({
  value,
  onChange,
  onAddTag,
  onSelectTag,
  currentTags,
  allTags,
  placeholder,
  maxLength = 50,
  inputId,
}: TagInputComboboxProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [searchQuery, setSearchQuery] = useState("")

  // 过滤已有标签：排除当前已添加的，并支持搜索
  const filteredTags = useMemo(() => {
    const availableTags = allTags.filter((tag) => !currentTags.includes(tag))
    if (!searchQuery.trim()) return availableTags
    const query = searchQuery.toLowerCase()
    return availableTags.filter((tag) => tag.toLowerCase().includes(query))
  }, [allTags, currentTags, searchQuery])

  const handleSelectTag = (tag: string) => {
    onSelectTag(tag)
    setOpen(false)
    setSearchQuery("")
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault()
      onAddTag()
    }
  }

  // 没有可选标签时隐藏下拉按钮
  const hasAvailableTags = allTags.length > 0

  return (
    <div className="flex gap-2">
      <Input
        id={inputId}
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        maxLength={maxLength}
        className="flex-1"
      />
      <Button
        onClick={onAddTag}
        size="sm"
        disabled={!value.trim()}
        aria-label={t("domain.tags.inputLabel")}
      >
        <Plus className="h-4 w-4" />
      </Button>

      {hasAvailableTags && (
        <Popover
          open={open}
          onOpenChange={(newOpen) => {
            setOpen(newOpen)
            if (!newOpen) setSearchQuery("")
          }}
        >
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              size="sm"
              className="px-2"
              aria-label={t("domain.tags.selectExisting")}
            >
              <ChevronDown className={cn("h-4 w-4 transition-transform", open && "rotate-180")} />
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-56 p-0" align="end">
            <Command>
              <CommandInput
                placeholder={t("domain.tags.searchExisting")}
                value={searchQuery}
                onValueChange={setSearchQuery}
              />
              <CommandList className="max-h-[200px]">
                <CommandEmpty>{t("domain.tags.noExistingTags")}</CommandEmpty>
                <CommandGroup heading={t("domain.tags.selectExisting")}>
                  {filteredTags.map((tag) => (
                    <CommandItem key={tag} onSelect={() => handleSelectTag(tag)}>
                      <span className="truncate">{tag}</span>
                    </CommandItem>
                  ))}
                </CommandGroup>
              </CommandList>
            </Command>
          </PopoverContent>
        </Popover>
      )}
    </div>
  )
}
