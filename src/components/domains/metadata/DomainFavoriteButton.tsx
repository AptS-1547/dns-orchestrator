import { Star } from "lucide-react"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { useDomainStore } from "@/stores/domainStore"

interface DomainFavoriteButtonProps {
  accountId: string
  domainId: string
  isFavorite: boolean
}

export function DomainFavoriteButton({
  accountId,
  domainId,
  isFavorite,
}: DomainFavoriteButtonProps) {
  const toggleFavorite = useDomainStore((state) => state.toggleFavorite)

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation() // 阻止事件冒泡（避免触发域名选择）
    toggleFavorite(accountId, domainId)
  }

  return (
    <Button
      variant="ghost"
      size="icon"
      onClick={handleClick}
      className="h-8 w-8"
      title={isFavorite ? "取消收藏" : "收藏"}
    >
      <Star
        className={cn(
          "h-4 w-4 transition-colors",
          isFavorite ? "fill-yellow-400 text-yellow-400" : "text-muted-foreground"
        )}
      />
    </Button>
  )
}
