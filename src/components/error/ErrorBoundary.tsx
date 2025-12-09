import { AlertTriangle, RefreshCw } from "lucide-react"
import { Component, type ErrorInfo, type ReactNode } from "react"
import { Button } from "@/components/ui/button"
import i18n from "@/i18n"
import { ENV } from "@/lib/env"

interface Props {
  children: ReactNode
  /** 错误回调 */
  onError?: (error: Error, errorInfo: ErrorInfo) => void
  /** 是否显示重试按钮 */
  showRetry?: boolean
  /** 自定义重试处理 */
  onRetry?: () => void
  /** 回退 UI 的最小高度 */
  minHeight?: string
  /** 边界级别：global/page/component */
  level?: "global" | "page" | "component"
  /** 边界名称，用于日志 */
  name?: string
}

interface State {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props)
    this.state = { hasError: false, error: null }
  }

  static getDerivedStateFromError(error: Error): Partial<State> {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    const { onError, level = "component", name = "unknown" } = this.props
    console.error(`[ErrorBoundary:${level}:${name}]`, error, errorInfo)
    onError?.(error, errorInfo)
  }

  handleRetry = () => {
    const { onRetry } = this.props
    this.setState({ hasError: false, error: null })
    onRetry?.()
  }

  handleReload = () => {
    window.location.reload()
  }

  render() {
    const { hasError, error } = this.state
    const { children, showRetry = true, minHeight = "200px", level } = this.props

    if (!hasError) {
      return children
    }

    const t = i18n.t.bind(i18n)
    const isGlobal = level === "global"

    return (
      <div
        className={`flex flex-col items-center justify-center gap-4 p-6 ${
          isGlobal ? "h-screen" : ""
        }`}
        style={{ minHeight: isGlobal ? undefined : minHeight }}
      >
        <AlertTriangle className={`text-destructive ${isGlobal ? "h-16 w-16" : "h-10 w-10"}`} />
        <div className="space-y-2 text-center">
          <h3 className={`font-semibold ${isGlobal ? "text-xl" : "text-lg"}`}>
            {t("error.title")}
          </h3>
          <p className="max-w-md text-muted-foreground text-sm">{t("error.description")}</p>
          {ENV.isDev && error && (
            <pre className="mt-4 max-w-lg overflow-auto rounded bg-muted p-3 text-left text-xs">
              {error.message}
              {error.stack && `\n\n${error.stack}`}
            </pre>
          )}
        </div>
        <div className="flex gap-2">
          {showRetry && (
            <Button variant="outline" onClick={this.handleRetry}>
              <RefreshCw className="mr-2 h-4 w-4" />
              {t("error.retry")}
            </Button>
          )}
          {isGlobal && (
            <Button variant="ghost" size="sm" onClick={this.handleReload}>
              {t("error.reload")}
            </Button>
          )}
        </div>
      </div>
    )
  }
}
