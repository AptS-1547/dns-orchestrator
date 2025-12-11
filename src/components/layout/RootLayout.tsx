/**
 * 根布局组件
 * 包含初始化逻辑、全局布局和路由出口
 */

import { useEffect } from "react"
import { Outlet, useLocation, useNavigate } from "react-router-dom"
import { ErrorBoundary } from "@/components/error"
import { Toaster } from "@/components/ui/sonner"
import { StatusBar } from "@/components/ui/status-bar"
import { UpdateDialog } from "@/components/ui/update-dialog"
import { TIMING } from "@/constants"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { initDebugMode } from "@/lib/debug"
import { logger } from "@/lib/logger"
import { initTheme, useAccountStore, useDomainStore } from "@/stores"
import { useUpdaterStore } from "@/stores/updaterStore"
import { AppLayout } from "./AppLayout"

/** 路由路径到导航项的映射 */
function getNavItemFromPath(
  pathname: string
): "main" | "domains" | "toolbox" | "settings" | "accounts" {
  if (pathname === "/") return "main"
  if (pathname.startsWith("/domains")) return "domains"
  if (pathname.startsWith("/toolbox")) return "toolbox"
  if (pathname.startsWith("/settings")) return "settings"
  if (pathname.startsWith("/accounts")) return "accounts"
  return "main"
}

export function RootLayout() {
  const location = useLocation()
  const navigate = useNavigate()
  const isMobile = useIsMobile()

  const { checkForUpdates, showUpdateDialog, setShowUpdateDialog } = useUpdaterStore()
  const { accounts, fetchAccounts } = useAccountStore()
  const { loadFromStorage, refreshAllAccounts } = useDomainStore()

  // 初始化
  useEffect(() => {
    initTheme()
    initDebugMode()
    fetchAccounts()
    loadFromStorage()
  }, [fetchAccounts, loadFromStorage])

  // 账户加载完成后，后台刷新域名
  useEffect(() => {
    if (accounts.length > 0) {
      const validAccounts = accounts.filter((a) => a.status !== "error")
      if (validAccounts.length > 0) {
        refreshAllAccounts(validAccounts)
      }
    }
  }, [accounts, refreshAllAccounts])

  // 检查更新（仅桌面端）
  useEffect(() => {
    if (isMobile) return

    const timer = setTimeout(async () => {
      try {
        await checkForUpdates()
      } catch (error) {
        logger.error("Update check failed:", error)
      }
    }, TIMING.UPDATE_CHECK_DELAY)

    return () => clearTimeout(timer)
  }, [checkForUpdates, isMobile])

  // 导航处理
  const handleNavigate = (view: "main" | "domains" | "toolbox" | "settings" | "accounts") => {
    const paths: Record<string, string> = {
      main: "/",
      domains: "/domains",
      toolbox: "/toolbox",
      settings: "/settings",
      accounts: "/accounts",
    }
    navigate(paths[view])
  }

  // 当前导航项（用于侧边栏高亮）
  const currentNavItem = getNavItemFromPath(location.pathname)

  // 移动端子页面时隐藏 AppLayout 的 header
  const shouldHideHeader = isMobile && location.pathname !== "/"

  return (
    <ErrorBoundary level="global" name="App" showRetry={false}>
      <AppLayout
        currentView={currentNavItem}
        onNavigate={handleNavigate}
        hideHeader={shouldHideHeader}
      >
        <ErrorBoundary level="page" name="ContentArea">
          <Outlet />
        </ErrorBoundary>
      </AppLayout>
      {!isMobile && <StatusBar />}
      <UpdateDialog open={showUpdateDialog} onOpenChange={setShowUpdateDialog} />
      <Toaster richColors position={isMobile ? "bottom-center" : "top-right"} />
    </ErrorBoundary>
  )
}
