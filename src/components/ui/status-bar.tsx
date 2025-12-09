import { Check, Download, Loader2, RefreshCw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { cn } from "@/lib/utils"
import { useUpdaterStore } from "@/stores/updaterStore"

type StatusType = "idle" | "checking" | "available" | "downloading" | "retrying" | "installing"

export function StatusBar() {
  const { t } = useTranslation()
  const {
    checking,
    downloading,
    progress,
    available,
    retryCount,
    maxRetries,
    setShowUpdateDialog,
  } = useUpdaterStore()

  // 获取当前状态类型
  const getStatusType = (): StatusType => {
    if (progress >= 100 && downloading) return "installing"
    if (retryCount > 0 && downloading) return "retrying"
    if (downloading) return "downloading"
    if (available) return "available"
    if (checking) return "checking"
    return "idle"
  }

  const statusType = getStatusType()

  // 左侧是否可点击（有更新时可点击打开对话框）
  const isLeftClickable = statusType === "available"

  // 处理点击更新 - 打开对话框
  const handleClickUpdate = () => {
    if (!isLeftClickable) return
    setShowUpdateDialog(true)
  }

  // 获取左侧图标
  const getStatusIcon = () => {
    switch (statusType) {
      case "idle":
        return <Check className="h-3.5 w-3.5" />
      case "checking":
      case "installing":
        return <Loader2 className="h-3.5 w-3.5 animate-spin" />
      case "available":
      case "downloading":
        return <Download className="h-3.5 w-3.5" />
      case "retrying":
        return <RefreshCw className="h-3.5 w-3.5 animate-spin" />
    }
  }

  // 获取左侧文本
  const getStatusText = () => {
    switch (statusType) {
      case "idle":
        return t("statusBar.ready")
      case "checking":
        return t("statusBar.checking")
      case "available":
        return `${t("statusBar.available")} v${available?.version}`
      case "downloading":
        return `${t("statusBar.downloading")} ${progress}%`
      case "retrying":
        return `${t("statusBar.retrying")} (${retryCount}/${maxRetries})`
      case "installing":
        return t("statusBar.installing")
    }
  }

  // 获取背景样式
  const getBackgroundClass = () => {
    switch (statusType) {
      case "available":
        return "bg-primary/10 text-primary border-primary/20"
      case "retrying":
        return "bg-yellow-500/20 border-yellow-500/50 text-foreground"
      default:
        return "bg-muted text-muted-foreground"
    }
  }

  return (
    <footer className={cn("fixed right-0 bottom-0 left-0 z-50 h-6 border-t", getBackgroundClass())}>
      {/* 进度条（下载时显示在顶部） */}
      {downloading && statusType !== "installing" && (
        <div className="absolute top-0 right-0 left-0 h-0.5 bg-muted/50">
          <div
            className={cn(
              "h-full transition-all duration-300 ease-out",
              statusType === "retrying" ? "bg-yellow-500" : "bg-primary"
            )}
            style={{ width: `${progress}%` }}
          />
        </div>
      )}

      {/* 状态栏内容 */}
      <div className="flex h-full items-center justify-between px-3 text-xs">
        {/* 左侧状态区（有更新时可点击下载） */}
        {isLeftClickable ? (
          <button
            type="button"
            onClick={handleClickUpdate}
            className="-ml-2 flex h-full items-center gap-1.5 rounded px-2 transition-colors hover:bg-primary/20"
          >
            {getStatusIcon()}
            <span>{getStatusText()}</span>
            <span className="opacity-70">({t("statusBar.clickToUpdate")})</span>
          </button>
        ) : (
          <div className="flex items-center gap-1.5">
            {getStatusIcon()}
            <span>{getStatusText()}</span>
          </div>
        )}

        {/* 右侧版本号 */}
        <span className="text-muted-foreground tabular-nums">v{__APP_VERSION__}</span>
      </div>
    </footer>
  )
}
