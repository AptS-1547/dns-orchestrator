import { Loader2, Search, Server } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
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
import { DNS_SERVERS } from "@/constants"
import { useEnterKeyHandler } from "@/hooks/useEnterKeyHandler"
import type { DnsLookupResult, DnsLookupType } from "@/types"
import { DNS_RECORD_TYPES } from "@/types"
import { HistoryChips } from "./HistoryChips"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { CopyableText, ToolCard } from "./shared"

export function DnsLookup() {
  const { t } = useTranslation()
  const [domain, setDomain] = useState("")
  const [recordType, setRecordType] = useState<DnsLookupType>("ALL")
  const [dnsServer, setDnsServer] = useState("system")
  const [customDns, setCustomDns] = useState("")

  const { isLoading, result, execute } = useToolboxQuery<DnsLookupResult>()

  const handleLookup = async () => {
    const trimmed = domain.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterDomain"))
      return
    }

    // 计算实际使用的 nameserver
    let nameserver: string | null = null
    if (dnsServer === "custom") {
      if (!customDns.trim()) {
        toast.error(t("toolbox.enterCustomDns"))
        return
      }
      nameserver = customDns.trim()
    } else if (dnsServer !== "system") {
      nameserver = dnsServer
    }

    const data = await execute(() => toolboxService.dnsLookup(trimmed, recordType, nameserver), {
      type: "dns",
      query: trimmed,
      recordType,
    })
    if (data && data.records.length === 0) {
      toast.info(t("toolbox.noRecords"))
    }
  }

  const handleKeyDown = useEnterKeyHandler(handleLookup)

  const records = result?.records ?? []

  return (
    <ToolCard title={t("toolbox.dnsLookup")}>
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
              {DNS_RECORD_TYPES.map((type) => (
                <SelectItem key={type} value={type}>
                  {type}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <Button onClick={handleLookup} disabled={isLoading} className="w-full sm:w-auto">
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <Search className="h-4 w-4" />
          )}
          <span className="ml-2">{t("toolbox.query")}</span>
        </Button>
      </div>

      {/* DNS 服务器选择 */}
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div className="flex items-center gap-2 text-muted-foreground text-sm">
          <Server className="h-4 w-4" />
          <span>{t("toolbox.dnsServer")}:</span>
        </div>
        <div className="flex flex-1 flex-col gap-2 sm:flex-row">
          <Select value={dnsServer} onValueChange={setDnsServer} disabled={isLoading}>
            <SelectTrigger className="w-full sm:w-48">
              <SelectValue placeholder={t("toolbox.systemDefault")} />
            </SelectTrigger>
            <SelectContent>
              {DNS_SERVERS.map((server) => (
                <SelectItem key={server.value} value={server.value}>
                  {server.isRaw ? server.label : t(server.label)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {dnsServer === "custom" && (
            <Input
              placeholder={t("toolbox.customDnsPlaceholder")}
              value={customDns}
              onChange={(e) => setCustomDns(e.target.value)}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              className="flex-1"
            />
          )}
        </div>
      </div>

      {/* 历史记录 */}
      <HistoryChips
        type="dns"
        onSelect={(item) => {
          setDomain(item.query)
          if (item.recordType) {
            setRecordType(item.recordType as DnsLookupType)
          }
        }}
      />

      {/* 显示使用的 DNS 服务器 */}
      {result && (
        <div className="flex items-center gap-2 text-muted-foreground text-sm">
          <Server className="h-4 w-4" />
          <span>
            {t("toolbox.usedDnsServer")}: <span className="font-mono">{result.nameserver}</span>
          </span>
        </div>
      )}

      {/* 查询结果 - 移动端卡片 */}
      {records.length > 0 && (
        <div className="space-y-2 sm:hidden">
          {records.map((record, index) => (
            <CopyableText
              key={`${record.recordType}-${record.name}-${record.value}-${index}`}
              value={record.value}
              className="block"
            >
              <div className="rounded-lg border bg-card p-3">
                <div className="mb-2 flex items-center gap-2">
                  <span className="rounded bg-primary/10 px-2 py-0.5 font-medium text-primary text-xs">
                    {record.recordType}
                  </span>
                  <span className="text-muted-foreground text-xs">TTL: {record.ttl}</span>
                  {record.priority != null && (
                    <span className="text-muted-foreground text-xs">
                      {t("dns.priority")}: {record.priority}
                    </span>
                  )}
                </div>
                <div className="break-all font-mono text-sm">{record.name}</div>
                <div className="break-all font-mono text-muted-foreground text-sm">
                  {record.value}
                </div>
              </div>
            </CopyableText>
          ))}
        </div>
      )}

      {/* 查询结果 - 桌面端 Table */}
      {records.length > 0 && (
        <div className="hidden rounded-md border sm:block">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-20">{t("common.type")}</TableHead>
                <TableHead>{t("dns.name")}</TableHead>
                <TableHead>{t("dns.value")}</TableHead>
                <TableHead className="w-20">{t("dns.ttl")}</TableHead>
                <TableHead className="w-20">{t("dns.priority")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {records.map((record, index) => (
                <TableRow key={`${record.recordType}-${record.name}-${record.value}-${index}`}>
                  <TableCell>
                    <span className="rounded bg-primary/10 px-2 py-0.5 font-medium text-primary text-xs">
                      {record.recordType}
                    </span>
                  </TableCell>
                  <TableCell className="w-48 font-mono text-sm">{record.name}</TableCell>
                  <TableCell className="max-w-0 font-mono text-sm">
                    <CopyableText value={record.value} className="block truncate" />
                  </TableCell>
                  <TableCell className="text-muted-foreground">{record.ttl}</TableCell>
                  <TableCell className="text-muted-foreground">{record.priority ?? "-"}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}
    </ToolCard>
  )
}
