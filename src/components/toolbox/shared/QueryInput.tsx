import { Loader2, Search } from "lucide-react"
import { memo, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import type { QueryHistoryItem } from "@/types"
import { HistoryChips } from "../HistoryChips"

interface QueryInputProps {
  /** 输入值 */
  value: string
  /** 值变化回调 */
  onChange: (value: string) => void
  /** 提交回调 */
  onSubmit: () => void
  /** 是否正在加载 */
  isLoading: boolean
  /** 输入框占位符 */
  placeholder?: string
  /** 历史记录类型 */
  historyType: QueryHistoryItem["type"]
  /** 历史记录选中回调 */
  onHistorySelect: (item: QueryHistoryItem) => void
  /** 额外的输入控件，渲染在输入框和按钮之间 */
  extraInput?: ReactNode
  /** 内嵌控件，渲染在输入框边框内部（如 Select、端口输入） */
  inlineExtra?: ReactNode
  /** 自定义按钮文本 */
  buttonText?: string
  /** 输入行与历史记录之间的内容 */
  belowInput?: ReactNode
}

/**
 * 查询输入区组件
 * 包含：输入框 + 额外输入 + 查询按钮 + 历史记录
 */
function QueryInputComponent({
  value,
  onChange,
  onSubmit,
  isLoading,
  placeholder,
  historyType,
  onHistorySelect,
  extraInput,
  inlineExtra,
  buttonText,
  belowInput,
}: QueryInputProps) {
  const { t } = useTranslation()

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      onSubmit()
    }
  }

  return (
    <>
      <div className="flex flex-col gap-2 sm:flex-row">
        {inlineExtra ? (
          <div className="flex flex-1 items-center rounded-md border bg-background">
            <Input
              placeholder={placeholder}
              value={value}
              onChange={(e) => onChange(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              className="flex-1 border-0 shadow-none"
            />
            {inlineExtra}
          </div>
        ) : (
          <Input
            placeholder={placeholder}
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading}
            className="flex-1"
          />
        )}
        {extraInput}
        <Button onClick={onSubmit} disabled={isLoading} className="w-full sm:w-auto">
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Search className="h-4 w-4" />
          )}
          <span className="ml-2">{buttonText ?? t("toolbox.query")}</span>
        </Button>
      </div>
      {belowInput}
      <HistoryChips type={historyType} onSelect={onHistorySelect} />
    </>
  )
}

export const QueryInput = memo(QueryInputComponent)
