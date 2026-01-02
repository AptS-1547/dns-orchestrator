/**
 * 移动端菜单触发器
 * 用于第一层子页面的 Header，点击打开侧边栏
 */

import { Menu } from "lucide-react"
import { useState } from "react"
import { useLocation, useNavigate } from "react-router-dom"
import { Button } from "@/components/ui/button"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { getNavItemFromPath, NAV_PATHS, type NavItem } from "@/types"
import { Sidebar } from "./Sidebar"

export function MobileMenuTrigger() {
  const [open, setOpen] = useState(false)
  const location = useLocation()
  const navigate = useNavigate()
  const currentView = getNavItemFromPath(location.pathname)

  const handleNavigate = (view: NavItem) => {
    setOpen(false)
    navigate(NAV_PATHS[view])
  }

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>
        <Button variant="ghost" size="icon">
          <Menu className="h-5 w-5" />
        </Button>
      </SheetTrigger>
      <SheetContent side="left" className="w-72 bg-sidebar px-0" hideClose>
        <Sidebar
          currentView={currentView}
          onNavigate={handleNavigate}
          onClose={() => setOpen(false)}
          isMobile
        />
      </SheetContent>
    </Sheet>
  )
}
