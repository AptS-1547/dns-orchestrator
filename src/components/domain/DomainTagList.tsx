import { X } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { cn } from "@/lib/utils"

interface DomainTagListProps {
  tags: string[]
  /** 是否可编辑（显示删除按钮） */
  editable?: boolean
  /** 删除标签回调 */
  onRemoveTag?: (tag: string) => void
  /** 点击标签回调（用于筛选） */
  onClickTag?: (tag: string) => void
  /** 自定义样式 */
  className?: string
  /** 最多显示几个标签，超出显示 +N */
  maxDisplay?: number
}

export function DomainTagList({
  tags,
  editable = false,
  onRemoveTag,
  onClickTag,
  className,
  maxDisplay,
}: DomainTagListProps) {
  if (tags.length === 0) {
    return null
  }

  const displayTags = maxDisplay ? tags.slice(0, maxDisplay) : tags
  const remainingCount = maxDisplay ? Math.max(0, tags.length - maxDisplay) : 0

  return (
    <TooltipProvider>
      <div className={cn("flex flex-wrap gap-1.5", className)}>
        {displayTags.map((tag) => {
          const isLongTag = tag.length > 15

          const BadgeContent = (
            <Badge
              key={tag}
              variant={onClickTag ? "outline" : "secondary"}
              className={cn(
                "group relative transition-all",
                onClickTag && "cursor-pointer hover:border-primary/50 hover:bg-accent",
                editable && "pr-6"
              )}
              onClick={(e) => {
                e.stopPropagation()
                onClickTag?.(tag)
              }}
            >
              <span className="inline-block max-w-[120px] truncate align-bottom text-xs sm:max-w-[150px]">
                {tag}
              </span>
              {editable && onRemoveTag && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="absolute top-0 right-0 h-full w-5 p-0 opacity-100 transition-opacity [@media(hover:hover)]:opacity-0 [@media(hover:hover)]:group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation()
                    onRemoveTag(tag)
                  }}
                  aria-label={`Remove ${tag}`}
                >
                  <X className="h-3 w-3" />
                </Button>
              )}
            </Badge>
          )

          // 长标签包裹 Tooltip
          if (isLongTag) {
            return (
              <Tooltip key={tag}>
                <TooltipTrigger asChild>{BadgeContent}</TooltipTrigger>
                <TooltipContent side="top" className="max-w-[300px]">
                  {tag}
                </TooltipContent>
              </Tooltip>
            )
          }

          return BadgeContent
        })}
        {remainingCount > 0 && (
          <Tooltip>
            <TooltipTrigger asChild>
              <Badge variant="outline" className="text-xs">
                +{remainingCount}
              </Badge>
            </TooltipTrigger>
            <TooltipContent side="top" className="max-w-[300px]">
              {tags.slice(maxDisplay).join(", ")}
            </TooltipContent>
          </Tooltip>
        )}
      </div>
    </TooltipProvider>
  )
}
