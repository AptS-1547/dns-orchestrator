import { memo, type ReactNode } from "react"

interface InfoCardProps {
  /** 图标 */
  icon: ReactNode
  /** 标题 */
  title: string
  /** 子内容 */
  children: ReactNode
  /** 额外的 className */
  className?: string
}

/**
 * 信息展示卡片组件
 * 用于展示带图标和标题的信息区块
 */
function InfoCardComponent({ icon, title, children, className = "" }: InfoCardProps) {
  return (
    <div className={`rounded-lg border bg-card p-4 ${className}`}>
      <div className="mb-3 flex items-center gap-2">
        <span className="text-primary">{icon}</span>
        <span className="font-medium">{title}</span>
      </div>
      {children}
    </div>
  )
}

export const InfoCard = memo(InfoCardComponent)
