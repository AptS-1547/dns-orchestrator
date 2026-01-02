import { Globe, Menu } from "lucide-react"
import type { ReactNode } from "react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores"
import type { NavItem } from "@/types"
import { Sidebar } from "./Sidebar"

interface AppLayoutProps {
  children?: ReactNode
  currentView: NavItem
  onNavigate: (view: NavItem) => void
  /** 是否隐藏移动端 header（子页面自己负责显示） */
  hideHeader?: boolean
}

export function AppLayout({
  children,
  currentView,
  onNavigate,
  hideHeader = false,
}: AppLayoutProps) {
  const { t } = useTranslation()
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const { sidebarCollapsed } = useSettingsStore()

  return (
    <div className="flex h-screen w-full flex-col overflow-hidden bg-background md:flex-row md:pb-6">
      {/* 移动端顶部导航 */}
      {!hideHeader && (
        <header className="flex items-center gap-2 border-b px-4 py-3 md:hidden">
          <Sheet open={sidebarOpen} onOpenChange={setSidebarOpen}>
            <SheetTrigger asChild>
              <Button variant="ghost" size="icon">
                <Menu className="h-5 w-5" />
              </Button>
            </SheetTrigger>
            <SheetContent side="left" className="w-72 bg-sidebar px-0" hideClose>
              <Sidebar
                currentView={currentView}
                onNavigate={(view) => {
                  setSidebarOpen(false)
                  onNavigate(view)
                }}
                onClose={() => setSidebarOpen(false)}
                isMobile
              />
            </SheetContent>
          </Sheet>
          <Globe className="h-5 w-5 text-primary" />
          <h1 className="font-semibold text-xl">{t("common.appName")}</h1>
        </header>
      )}

      {/* 桌面端侧边栏 */}
      <div className={cn("hidden h-full shrink-0 md:block", sidebarCollapsed ? "w-16" : "w-56")}>
        <Sidebar currentView={currentView} onNavigate={onNavigate} />
      </div>

      {/* 主内容区 */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">{children}</div>
    </div>
  )
}
