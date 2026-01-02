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
    <div className="rounded-md border p-3 transition-colors hover:bg-muted/50">
      {/* Header name + Copy 按钮 */}
      <div className="mb-2 flex items-center justify-between gap-2">
        <code className="font-mono font-semibold text-sm">{header.name}</code>
        <Button variant="ghost" size="sm" onClick={onCopy} className="h-7">
          <Copy className="mr-1 size-3.5" />
          <span className="text-xs">{t("toolbox.httpHeaderCheck.copyHeader")}</span>
        </Button>
      </div>

      {/* Header value */}
      {isLongValue ? (
        <Collapsible className="w-full" open={isExpanded} onOpenChange={onToggle}>
          {!isExpanded && (
            <div className="max-w-full break-all font-mono text-muted-foreground text-sm">
              {header.value.slice(0, 80)}...
            </div>
          )}
          <CollapsibleTrigger asChild>
            <Button variant="ghost" size="sm" className="mt-2 h-7 text-xs">
              {isExpanded ? (
                <>
                  <ChevronUp className="mr-1 size-3.5 transition-transform duration-200" />
                  {t("toolbox.httpHeaderCheck.showLess")}
                </>
              ) : (
                <>
                  <ChevronDown className="mr-1 size-3.5 transition-transform duration-200" />
                  {t("toolbox.httpHeaderCheck.showFullValue")}
                </>
              )}
            </Button>
          </CollapsibleTrigger>
          <CollapsibleContent>
            <pre className="mt-2 max-w-full overflow-hidden whitespace-pre-wrap break-words rounded bg-muted p-2 font-mono text-muted-foreground text-sm">
              {header.value}
            </pre>
          </CollapsibleContent>
        </Collapsible>
      ) : (
        <div className="max-w-full break-all font-mono text-muted-foreground text-sm">
          {header.value}
        </div>
      )}
    </div>
  )
}

// 使用 memo 包装，避免不必要的重渲染
export const HeaderItem = memo(HeaderItemComponent)
