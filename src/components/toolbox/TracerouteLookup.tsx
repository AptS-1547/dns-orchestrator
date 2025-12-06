import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { useToolboxStore } from "@/stores"
import type { TracerouteHop, TracerouteProgress } from "@/types"
import { Channel, invoke } from "@tauri-apps/api/core"
import { Loader2, Network, Search, XCircle } from "lucide-react"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { HistoryChips } from "./HistoryChips"

export function TracerouteLookup() {
  const { t } = useTranslation()
  const { addHistory } = useToolboxStore()
  const [target, setTarget] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [hops, setHops] = useState<TracerouteHop[]>([])

  const handleTraceroute = useCallback(async () => {
    if (!target.trim()) {
      toast.error(t("toolbox.enterTarget"))
      return
    }

    setIsLoading(true)
    setHops([])

    try {
      // 创建 Channel 接收实时更新
      const onProgress = new Channel<TracerouteProgress>()

      onProgress.onmessage = (progress) => {
        if (progress.hop) {
          setHops((prev) => [...prev, progress.hop!])
        }
        if (progress.done) {
          setIsLoading(false)
          if (progress.error) {
            toast.error(progress.error)
          }
        }
      }

      await invoke("traceroute", {
        target: target.trim(),
        onProgress,
      })

      addHistory({
        type: "traceroute",
        query: target.trim(),
      })
    } catch (err) {
      toast.error(String(err))
      setIsLoading(false)
    }
  }, [target, addHistory, t])

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleTraceroute()
    }
  }

  const handleStop = useCallback(() => {
    // TODO: 实现停止功能（需要后端支持取消）
    setIsLoading(false)
    toast.info(t("toolbox.tracerouteStopped"))
  }, [t])

  // 格式化 RTT 显示
  const formatRtt = (rtt: number[]) => {
    if (rtt.length === 0) return "*"
    const avg = rtt.reduce((a, b) => a + b, 0) / rtt.length
    return `${avg.toFixed(2)} ms`
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          <Network className="h-5 w-5" />
          {t("toolbox.traceroute")}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* 查询输入 */}
        <div className="flex flex-col gap-2 sm:flex-row">
          <Input
            placeholder={t("toolbox.traceroutePlaceholder")}
            value={target}
            onChange={(e) => setTarget(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading}
            className="flex-1"
          />
          {isLoading ? (
            <Button variant="outline" onClick={handleStop} className="w-full sm:w-auto">
              <XCircle className="mr-2 h-4 w-4" />
              {t("common.stop")}
            </Button>
          ) : (
            <Button onClick={handleTraceroute} className="w-full sm:w-auto">
              <Search className="mr-2 h-4 w-4" />
              {t("toolbox.trace")}
            </Button>
          )}
        </div>

        {/* 历史记录 */}
        <HistoryChips
          type="traceroute"
          onSelect={(item) => {
            setTarget(item.query)
          }}
        />

        {/* 结果列表 */}
        {(hops.length > 0 || isLoading) && (
          <div className="space-y-1 rounded-lg border bg-card p-3 font-mono text-sm">
            {hops.map((hop, index) => (
              <div
                key={index}
                className="flex items-center gap-2 py-1"
              >
                <span className="w-6 text-right text-muted-foreground">{hop.hop}</span>
                <span className="flex-1">
                  {hop.ip ? (
                    <span
                      className="cursor-pointer hover:underline"
                      onClick={() => {
                        navigator.clipboard.writeText(hop.ip!)
                        toast.success(t("common.copied"))
                      }}
                    >
                      {hop.ip}
                    </span>
                  ) : (
                    <span className="text-muted-foreground">*</span>
                  )}
                </span>
                <span className={hop.rtt.length > 0 ? "text-green-600 dark:text-green-400" : "text-muted-foreground"}>
                  {formatRtt(hop.rtt)}
                </span>
              </div>
            ))}
            {isLoading && (
              <div className="flex items-center gap-2 py-1 text-muted-foreground">
                <Loader2 className="h-4 w-4 animate-spin" />
                <span>{t("toolbox.tracing")}</span>
              </div>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  )
}
