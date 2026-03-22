import type { ReactNode } from "react"
import { Button, type ButtonProps } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { useIsMobile } from "@/hooks/useMediaQuery"

interface ResponsiveButtonProps extends Omit<ButtonProps, "children" | "size"> {
  icon: ReactNode
  label: string
  tooltip?: string
}

/**
 * 响应式按钮：移动端显示图标 + tooltip，桌面端显示图标 + 文字
 */
export function ResponsiveButton({ icon, label, tooltip, ...buttonProps }: ResponsiveButtonProps) {
  const isMobile = useIsMobile()

  if (isMobile) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button size="icon" {...buttonProps}>
              {icon}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{tooltip ?? label}</TooltipContent>
        </Tooltip>
      </TooltipProvider>
    )
  }

  return (
    <Button size="sm" {...buttonProps}>
      {icon}
      <span className="ml-2">{label}</span>
    </Button>
  )
}
