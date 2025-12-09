import { Globe, Menu } from "lucide-react"
import type { ReactNode } from "react"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { cn } from "@/lib/utils"
import { MainContent } from "./MainContent"
import { Sidebar } from "./Sidebar"

interface AppLayoutProps {
  children?: ReactNode
  onOpenToolbox?: () => void
  onNavigateToMain?: () => void
  onOpenSettings?: () => void
  onOpenAccounts?: () => void
  /** 是否隐藏移动端 header（子页面自己负责显示） */
  hideHeader?: boolean
}

// 检测是否在 md-lg 之间（768px - 1024px）
function useIsCollapsedRange() {
  const [isCollapsed, setIsCollapsed] = useState(false)

  useEffect(() => {
    const mediaQuery = window.matchMedia("(min-width: 768px) and (max-width: 1023px)")
    const handleChange = (e: MediaQueryListEvent | MediaQueryList) => {
      setIsCollapsed(e.matches)
    }
    handleChange(mediaQuery)
    mediaQuery.addEventListener("change", handleChange)
    return () => mediaQuery.removeEventListener("change", handleChange)
  }, [])

  return isCollapsed
}

export function AppLayout({
  children,
  onOpenToolbox,
  onNavigateToMain,
  onOpenSettings,
  onOpenAccounts,
  hideHeader = false,
}: AppLayoutProps) {
  const { t } = useTranslation()
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const isCollapsedRange = useIsCollapsedRange()
  // Hover 状态
  const [hovered, setHovered] = useState(false)

  // 折叠状态：在折叠范围内且未 hover
  const collapsed = isCollapsedRange && !hovered

  return (
    <div className="flex h-screen w-full flex-col overflow-hidden bg-background md:flex-row md:pb-6">
      {/* 移动端顶部导航 - 根据 hideHeader 控制，子页面时隐藏 */}
      {!hideHeader && (
        <header className="flex items-center gap-2 border-b px-4 py-3 md:hidden">
          <Sheet open={sidebarOpen} onOpenChange={setSidebarOpen}>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon">
                <Menu className="h-5 w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-72 p-0" hideClose>
              <Sidebar
                onOpenToolbox={() => {
                  setSidebarOpen(false)
                  onOpenToolbox?.()
                }}
                onNavigateToMain={() => {
                  setSidebarOpen(false)
                  onNavigateToMain?.()
                }}
                onOpenSettings={() => {
                  setSidebarOpen(false)
                  onOpenSettings?.()
                }}
                onOpenAccounts={() => {
                  setSidebarOpen(false)
                  onOpenAccounts?.()
                }}
                onClose={() => setSidebarOpen(false)}
                isMobile
              />
            </SheetContent>
          </Sheet>
          <Globe className="h-5 w-5 text-primary" />
          <h1 className="font-semibold">{t("common.appName")}</h1>
        </header>
      )}

      {/* 桌面端侧边栏 - 仅桌面端显示 */}
      <div className="relative hidden h-full md:block">
        {/* 折叠时的占位容器 */}
        <div className={cn("h-full", isCollapsedRange ? "w-16" : "w-64")} />

        {/* 实际 Sidebar - 折叠范围内使用 absolute 定位实现浮层 */}
        <div
          role="navigation"
          className={cn(
            "h-full",
            isCollapsedRange ? "absolute inset-y-0 left-0 z-50" : "absolute inset-y-0 left-0"
          )}
          onMouseEnter={() => setHovered(true)}
          onMouseLeave={() => setHovered(false)}
        >
          <Sidebar
            onOpenToolbox={onOpenToolbox}
            onNavigateToMain={onNavigateToMain}
            onOpenSettings={onOpenSettings}
            onOpenAccounts={onOpenAccounts}
            collapsed={collapsed}
          />
          {/* 展开时的阴影 */}
          {isCollapsedRange && hovered && (
            <div className="-mr-4 pointer-events-none absolute inset-y-0 right-0 w-4 bg-gradient-to-r from-black/10 to-transparent" />
          )}
        </div>
      </div>

      {/* 主内容区 - 始终渲染，不会被卸载 */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        {children || <MainContent />}
      </div>
    </div>
  )
}
