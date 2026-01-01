import { AlertCircle, CheckCircle2, Clock, Loader2, Search, XCircle } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Progress } from "@/components/ui/progress"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useEnterKeyHandler } from "@/hooks/useEnterKeyHandler"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { cn } from "@/lib/utils"
import type { DnsPropagationResult, DnsLookupType } from "@/types"
import { DNS_RECORD_TYPES } from "@/types"
import { HistoryChips } from "./HistoryChips"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { CopyableText, ToolCard } from "./shared"

export function DnsPropagation() {
  const { t } = useTranslation()
  const isMobile = useIsMobile()
  const [domain, setDomain] = useState("")
  const [recordType, setRecordType] = useState<DnsLookupType>("A")

  const { isLoading, result, execute } = useToolboxQuery<DnsPropagationResult>()

  const handleCheck = async () => {
    const trimmed = domain.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterDomain"))
      return
    }

    const data = await execute(() => toolboxService.dnsPropagationCheck(trimmed, recordType), {
      type: "dns-propagation",
      query: trimmed,
      recordType,
    })

    if (data && data.results.length === 0) {
      toast.info(t("toolbox.dnsPropagation.noResults"))
    }
  }

  const handleKeyDown = useEnterKeyHandler(handleCheck)

  // 计算统计数据
  const stats = useMemo(() => {
    if (!result) return null

    const successful = result.results.filter((r) => r.status === "success").length
    const timeout = result.results.filter((r) => r.status === "timeout").length
    const error = result.results.filter((r) => r.status === "error").length
    const total = result.results.length

    return { successful, timeout, error, total }
  }, [result])

  // 获取一致性颜色
  const getConsistencyColor = (percentage: number) => {
    if (percentage >= 90) return "text-green-600"
    if (percentage >= 50) return "text-yellow-600"
    return "text-red-600"
  }

  // 获取状态图标
  const getStatusIcon = (status: string) => {
    switch (status) {
      case "success":
        return <CheckCircle2 className="h-4 w-4 text-green-600" />
      case "timeout":
        return <Clock className="h-4 w-4 text-yellow-600" />
      case "error":
        return <XCircle className="h-4 w-4 text-red-600" />
      default:
        return <AlertCircle className="h-4 w-4 text-gray-400" />
    }
  }

  // 获取状态文本
  const getStatusText = (status: string) => {
    return t(`toolbox.dnsPropagation.status.${status}`)
  }

  return (
    <ToolCard title={t("toolbox.dnsPropagation.title")}>
      {/* 查询输入 */}
      <div className="flex flex-col gap-2 sm:flex-row">
        <div className="flex flex-1 items-center rounded-md border bg-background">
          <Input
            placeholder={t("toolbox.domainPlaceholder")}
            value={domain}
            onChange={(e) => setDomain(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isLoading}
            className="flex-1 border-0 shadow-none"
          />
          <Select
            value={recordType}
            onValueChange={(v) => setRecordType(v as DnsLookupType)}
            disabled={isLoading}
          >
            <SelectTrigger className="w-auto gap-1 rounded-l-none border-0 border-l bg-transparent pr-3 pl-3 shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent className="max-h-60">
              {DNS_RECORD_TYPES.filter((t) => t !== "ALL").map((type) => (
                <SelectItem key={type} value={type}>
                  {type}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <Button onClick={handleCheck} disabled={isLoading} className="w-full sm:w-auto">
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Search className="h-4 w-4" />
          )}
          <span className="ml-2">{t("toolbox.dnsPropagation.check")}</span>
        </Button>
      </div>

      {/* 历史记录 */}
      <HistoryChips
        type="dns-propagation"
        onSelect={(item) => {
          setDomain(item.query)
          if (item.recordType) {
            setRecordType(item.recordType as DnsLookupType)
          }
        }}
      />

      {/* 摘要信息 */}
      {result && stats && (
        <div className="space-y-4 rounded-lg border bg-card p-4">
          {/* 一致性 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium">{t("toolbox.dnsPropagation.consistency")}</span>
              <span
                className={cn(
                  "text-2xl font-bold",
                  getConsistencyColor(result.consistencyPercentage)
                )}
              >
                {result.consistencyPercentage.toFixed(1)}%
              </span>
            </div>
            <Progress value={result.consistencyPercentage} className="h-2" />
          </div>

          {/* 统计 */}
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs">
                {t("toolbox.dnsPropagation.total")}
              </div>
              <div className="text-2xl font-bold">{stats.total}</div>
            </div>
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs">
                {t("toolbox.dnsPropagation.successful")}
              </div>
              <div className="text-2xl font-bold text-green-600">{stats.successful}</div>
            </div>
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs">
                {t("toolbox.dnsPropagation.timeout")}
              </div>
              <div className="text-2xl font-bold text-yellow-600">{stats.timeout}</div>
            </div>
            <div className="space-y-1">
              <div className="text-muted-foreground text-xs">
                {t("toolbox.dnsPropagation.failed")}
              </div>
              <div className="text-2xl font-bold text-red-600">{stats.error}</div>
            </div>
          </div>

          {/* 其他信息 */}
          <div className="flex flex-wrap gap-4 text-muted-foreground text-sm">
            <div>
              {t("toolbox.dnsPropagation.uniqueValues")}:{" "}
              <span className="font-mono font-medium">{result.uniqueValues.length}</span>
            </div>
            <div>
              {t("toolbox.dnsPropagation.totalTime")}:{" "}
              <span className="font-mono font-medium">{result.totalTimeMs} ms</span>
            </div>
          </div>
        </div>
      )}

      {/* 移动端: 卡片列表 */}
      {result && isMobile && (
        <div className="space-y-2">
          {result.results.map((serverResult) => (
            <div key={serverResult.server.ip} className="rounded-lg border bg-card p-3">
              <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  {getStatusIcon(serverResult.status)}
                  <span className="font-medium">{serverResult.server.name}</span>
                </div>
                <span className="text-muted-foreground text-xs">
                  {serverResult.responseTimeMs} ms
                </span>
              </div>
              <div className="mb-2 flex items-center gap-2 text-muted-foreground text-sm">
                <span>{serverResult.server.region}</span>
                <span>•</span>
                <span className="font-mono text-xs">{serverResult.server.ip}</span>
              </div>
              {serverResult.status === "success" && serverResult.records.length > 0 && (
                <div className="space-y-1">
                  {serverResult.records.map((record, idx) => (
                    <CopyableText
                      key={`${serverResult.server.ip}-${record.recordType}-${idx}`}
                      value={record.value}
                      className="block"
                    >
                      <div className="break-all font-mono text-sm">{record.value}</div>
                    </CopyableText>
                  ))}
                </div>
              )}
              {serverResult.error && (
                <div className="text-red-600 text-sm">{serverResult.error}</div>
              )}
            </div>
          ))}
        </div>
      )}

      {/* 桌面端: 表格 */}
      {result && !isMobile && (
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-12">{t("toolbox.dnsPropagation.statusLabel")}</TableHead>
                <TableHead className="w-36">{t("toolbox.dnsPropagation.server")}</TableHead>
                <TableHead className="w-32">{t("toolbox.dnsPropagation.region")}</TableHead>
                <TableHead className="w-28">IP</TableHead>
                <TableHead>{t("toolbox.dnsPropagation.result")}</TableHead>
                <TableHead className="w-20">TTL</TableHead>
                <TableHead className="w-24">{t("toolbox.dnsPropagation.responseTime")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {result.results.map((serverResult) => (
                <TableRow key={serverResult.server.ip}>
                  <TableCell>
                    <div className="flex items-center gap-1">
                      {getStatusIcon(serverResult.status)}
                    </div>
                  </TableCell>
                  <TableCell className="font-medium">{serverResult.server.name}</TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {serverResult.server.region}
                  </TableCell>
                  <TableCell className="font-mono text-sm">{serverResult.server.ip}</TableCell>
                  <TableCell>
                    {serverResult.status === "success" && serverResult.records.length > 0 ? (
                      <div className="space-y-1">
                        {serverResult.records.map((record, idx) => (
                          <CopyableText
                            key={`${serverResult.server.ip}-${record.recordType}-${idx}`}
                            value={record.value}
                            className="block truncate"
                          >
                            {record.value}
                          </CopyableText>
                        ))}
                      </div>
                    ) : serverResult.error ? (
                      <span className="text-red-600 text-sm">{serverResult.error}</span>
                    ) : (
                      <span className="text-muted-foreground text-sm">
                        {getStatusText(serverResult.status)}
                      </span>
                    )}
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {serverResult.records.length > 0 ? serverResult.records[0].ttl : "-"}
                  </TableCell>
                  <TableCell className="text-muted-foreground text-sm">
                    {serverResult.responseTimeMs} ms
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </ToolCard>
  )
}
