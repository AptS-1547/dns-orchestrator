import { ChevronDown, ChevronUp, Copy } from "lucide-react"
import { memo } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import type { HttpHeader } from "@/types"

interface HeaderItemProps {
  header: HttpHeader
  index: number
  isExpanded: boolean
  onToggle: (isOpen: boolean) => void
  onCopy: () => void
}

function HeaderItemComponent({ header, isExpanded, onToggle, onCopy }: HeaderItemProps) {
  const { t } = useTranslation()
  const isLongValue = header.value.length > 80

  return (
    <div className="border rounded-md p-3 hover:bg-muted/50 transition-colors">
      {/* Header name + Copy 按钮 */}
      <div className="flex items-center justify-between gap-2 mb-2">
        <code className="text-sm font-semibold font-mono">{header.name}</code>
        <Button variant="ghost" size="sm" onClick={onCopy} className="h-7">
          <Copy className="size-3.5 mr-1" />
          <span className="text-xs">{t("toolbox.httpHeaderCheck.copyHeader")}</span>
        </Button>
      </div>

      {/* Header value */}
      {isLongValue ? (
        <Collapsible className="w-full" open={isExpanded} onOpenChange={onToggle}>
          {!isExpanded && (
            <div className="text-sm text-muted-foreground font-mono break-all max-w-full">
              {header.value.slice(0, 80)}...
            </div>
          )}
          <CollapsibleTrigger asChild>
            <Button variant="ghost" size="sm" className="mt-2 h-7 text-xs">
              {isExpanded ? (
                <>
                  <ChevronUp className="size-3.5 mr-1 transition-transform duration-200" />
                  {t("toolbox.httpHeaderCheck.showLess")}
                </>
              ) : (
                <>
                  <ChevronDown className="size-3.5 mr-1 transition-transform duration-200" />
                  {t("toolbox.httpHeaderCheck.showFullValue")}
                </>
              )}
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <pre className="text-sm text-muted-foreground font-mono whitespace-pre-wrap break-words mt-2 p-2 bg-muted rounded max-w-full overflow-hidden">
              {header.value}
            </pre>
          </CollapsibleContent>
        </Collapsible>
      ) : (
        <div className="text-sm text-muted-foreground font-mono break-all max-w-full">
          {header.value}
        </div>
      )}
    </div>
  )
}

// 使用 memo 包装，避免不必要的重渲染
export const HeaderItem = memo(HeaderItemComponent)
