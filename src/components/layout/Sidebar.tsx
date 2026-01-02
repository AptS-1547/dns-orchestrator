import {
  Globe,
  Home,
  PanelLeftClose,
  PanelLeftOpen,
  Settings,
  Star,
  Users,
  Wrench,
} from "lucide-react"
import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { TIMING } from "@/constants"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores"
import type { NavItem } from "@/types"

interface SidebarProps {
  currentView: NavItem
  onNavigate: (view: NavItem) => void
  /** 是否为移动端模式 */
  isMobile?: boolean
  /** 关闭 Sidebar（移动端使用） */
  onClose?: () => void
}

interface NavItemConfig {
  id: NavItem
  icon: React.ComponentType<{ className?: string }>
  labelKey: string
  position: "top" | "bottom"
}

const navItems: NavItemConfig[] = [
  { id: "main", icon: Home, labelKey: "nav.home", position: "top" },
  { id: "domains", icon: Globe, labelKey: "nav.domains", position: "top" },
  { id: "favorites", icon: Star, labelKey: "nav.favorites", position: "top" },
  { id: "accounts", icon: Users, labelKey: "accounts.manage", position: "top" },
  { id: "toolbox", icon: Wrench, labelKey: "toolbox.title", position: "bottom" },
  { id: "settings", icon: Settings, labelKey: "settings.title", position: "bottom" },
]

export function Sidebar({ currentView, onNavigate, isMobile = false, onClose }: SidebarProps) {
  const { t } = useTranslation()
  const { sidebarCollapsed, setSidebarCollapsed } = useSettingsStore()

  // 移动端始终展开
  const collapsed = isMobile ? false : sidebarCollapsed

  const handleNavigate = (view: NavItem) => {
    onNavigate(view)
    if (isMobile) {
      onClose?.()
    }
  }

  const toggleCollapse = () => {
    setSidebarCollapsed(!sidebarCollapsed)
  }

  // navItems 是静态数组，useMemo 避免每次渲染重新计算
  const { topItems, bottomItems } = useMemo(
    () => ({
      topItems: navItems.filter((item) => item.position === "top"),
      bottomItems: navItems.filter((item) => item.position === "bottom"),
    }),
    []
  )

  const renderNavButton = (item: NavItemConfig) => {
    const Icon = item.icon
    const isActive = currentView === item.id
    const label = t(item.labelKey)

    const button = (
      <Button
        key={item.id}
        variant={isActive ? "secondary" : "ghost"}
        className={cn(
          "w-full justify-start gap-3",
          collapsed ? "h-10 w-10 justify-center p-0" : "h-10 px-3"
        )}
        onClick={() => handleNavigate(item.id)}
        aria-current={isActive ? "page" : undefined}
      >
        <Icon className="h-4 w-4 shrink-0" />
        {!collapsed && <span>{label}</span>}
      </Button>
    )

    if (collapsed) {
      return (
        <Tooltip key={item.id}>
          <TooltipTrigger asChild>{button}</TooltipTrigger>
          <TooltipContent side="right">{label}</TooltipContent>
        </Tooltip>
      )
    }

    return button
  }

  return (
    <TooltipProvider delayDuration={TIMING.TOOLTIP_DELAY}>
      <aside
        className={cn(
          "flex h-full flex-col border-r bg-sidebar transition-all duration-200",
          isMobile ? "w-full" : collapsed ? "w-16" : "w-56"
        )}
      >
        {/* Header */}
        <div
          className={cn(
            "flex items-center border-b",
            collapsed ? "justify-center p-3" : "justify-between px-4 py-3"
          )}
        >
          <div
            className={cn("flex items-center gap-2", collapsed && "justify-center")}
            onClick={() => handleNavigate("main")}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => e.key === "Enter" && handleNavigate("main")}
          >
            <Globe className="h-6 w-6 shrink-0 text-primary" />
            {!collapsed && (
              <span className="whitespace-nowrap font-semibold text-xl md:text-base">
                {t("common.appName")}
              </span>
            )}
          </div>

          {/* 折叠按钮 - 仅桌面端显示 */}
          {!(isMobile || collapsed) && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8" onClick={toggleCollapse}>
                  <PanelLeftClose className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">{t("nav.collapse")}</TooltipContent>
            </Tooltip>
          )}
        </div>

        {/* 主导航区 */}
        <nav className={cn("flex-1 space-y-1 p-2", collapsed && "flex flex-col items-center")}>
          {topItems.map(renderNavButton)}
        </nav>

        {/* 底部导航 */}
        <div className={cn("space-y-1 border-t p-2", collapsed && "flex flex-col items-center")}>
          {bottomItems.map(renderNavButton)}

          {/* 展开按钮 - 仅桌面端折叠时显示 */}
          {!isMobile && collapsed && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  className="h-10 w-10 justify-center p-0"
                  onClick={toggleCollapse}
                >
                  <PanelLeftOpen className="h-4 w-4" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">{t("nav.expand")}</TooltipContent>
            </Tooltip>
          )}
        </div>
      </aside>
    </TooltipProvider>
  )
}
