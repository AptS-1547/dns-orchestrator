import type * as React from "react"
import { cn } from "@/lib/utils"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./card"

interface SectionCardProps {
  /** 标题 */
  title: string
  /** 标题前的图标 */
  icon?: React.ReactNode
  /** 描述 */
  description?: string
  /** 子内容 */
  children: React.ReactNode
  /** 是否显示 header，默认 true */
  showHeader?: boolean
  /** 内容区域的额外 className */
  contentClassName?: string
  /** 卡片的额外 className */
  className?: string
}

/**
 * 区块卡片组件
 * 统一带标题的卡片布局
 */
export function SectionCard({
  title,
  icon,
  description,
  children,
  showHeader = true,
  contentClassName,
  className,
}: SectionCardProps) {
  if (!showHeader) {
    return (
      <Card className={className}>
        <CardContent className={cn("p-4", contentClassName)}>{children}</CardContent>
      </Card>
    )
  }

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          {icon}
          {title}
        </CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent className={cn("px-2 sm:px-4", contentClassName)}>{children}</CardContent>
    </Card>
  )
}
