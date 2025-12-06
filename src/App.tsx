import { AppLayout } from "@/components/layout/AppLayout"
import { SettingsPage } from "@/components/settings/SettingsPage"
import { ToolboxPage } from "@/components/toolbox/ToolboxPage"
import { Toaster } from "@/components/ui/sonner"
import { StatusBar } from "@/components/ui/status-bar"
import { UpdateDialog } from "@/components/ui/update-dialog"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { initDebugMode } from "@/lib/debug"
import { initTheme, useDomainStore } from "@/stores"
import { useUpdaterStore } from "@/stores/updaterStore"
import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"

type View = "main" | "settings" | "toolbox"

function App() {
  const { t } = useTranslation()
  const { checkForUpdates, showUpdateDialog, setShowUpdateDialog } = useUpdaterStore()
  const { selectDomain } = useDomainStore()
  const [currentView, setCurrentView] = useState<View>("main")
  const isMobile = useIsMobile()

  useEffect(() => {
    initTheme()
    initDebugMode()

    // 移动端不检查更新（Android 使用应用商店或手动更新）
    if (isMobile) return

    // 启动后延迟 3 秒检查更新（静默检查，状态栏会自动显示）
    const timer = setTimeout(async () => {
      try {
        await checkForUpdates()
      } catch (error) {
        // 启动时的错误检查不打扰用户，仅记录日志
        console.error("Update check failed:", error)
      }
    }, 3000)

    return () => clearTimeout(timer)
  }, [checkForUpdates, t, isMobile])

  const handleOpenSettings = () => {
    selectDomain(null)
    setCurrentView("settings")
  }

  return (
    <>
      <AppLayout
        onOpenToolbox={() => {
          selectDomain(null)
          setCurrentView("toolbox")
        }}
        onNavigateToMain={() => setCurrentView("main")}
        onOpenSettings={handleOpenSettings}
      >
        {currentView === "settings" ? (
          <SettingsPage onBack={() => setCurrentView("main")} />
        ) : currentView === "toolbox" ? (
          <ToolboxPage onBack={() => setCurrentView("main")} />
        ) : null}
      </AppLayout>
      {/* 桌面端显示底部状态栏 */}
      {!isMobile && <StatusBar onOpenSettings={handleOpenSettings} />}
      {/* 更新对话框 */}
      <UpdateDialog open={showUpdateDialog} onOpenChange={setShowUpdateDialog} />
      {/* Toast 位置：移动端底部居中，桌面端右上角 */}
      <Toaster richColors position={isMobile ? "bottom-center" : "top-right"} />
    </>
  )
}

export default App
