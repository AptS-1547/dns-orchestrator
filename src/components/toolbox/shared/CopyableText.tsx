import { memo, type ReactNode } from "react"
import { useCopyToClipboard } from "@/hooks/useCopyToClipboard"

interface CopyableTextProps {
  /** 要复制的值 */
  value: string
  /** 显示的内容（默认为 value） */
  children?: ReactNode
  /** 额外的 className */
  className?: string
}

/**
 * 可复制文本组件
 * 点击后复制到剪贴板并显示成功提示
 */
function CopyableTextComponent({ value, children, className = "" }: CopyableTextProps) {
  const copyToClipboard = useCopyToClipboard()

  const handleClick = () => {
    copyToClipboard(value)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" || e.key === " ") {
      handleClick()
    }
  }

  return (
    <span
      className={`cursor-pointer hover:underline ${className}`}
      onClick={handleClick}
      onKeyDown={handleKeyDown}
      role="button"
      tabIndex={0}
    >
      {children ?? value}
    </span>
  )
}

export const CopyableText = memo(CopyableTextComponent)
