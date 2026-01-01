import { memo, type ReactNode } from "react"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"

interface ToolCardProps {
  /** 卡片标题 */
  title: string
  /** 子内容 */
  children: ReactNode
}

/**
 * 工具卡片容器组件
 * 统一工具箱各工具的卡片布局
 */
function ToolCardComponent({ title, children }: ToolCardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">{title}</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">{children}</CardContent>
    </Card>
  )
}

export const ToolCard = memo(ToolCardComponent)
