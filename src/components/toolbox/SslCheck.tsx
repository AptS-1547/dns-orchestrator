import {
  AlertTriangle,
  CheckCircle,
  ChevronDown,
  Clock,
  Globe,
  Link,
  Loader2,
  Lock,
  Search,
  Shield,
  Unlock,
  XCircle,
} from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Input } from "@/components/ui/input"
import { NETWORK } from "@/constants"
import { useEnterKeyHandler } from "@/hooks/useEnterKeyHandler"
import type { SslCheckResult } from "@/types"
import { HistoryChips } from "./HistoryChips"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { CopyableText, InfoCard, ToolCard } from "./shared"

/** 获取状态信息 */
function getStatusInfo(result: SslCheckResult | null, t: (key: string) => string) {
  if (!result) return null

  if (result.connectionStatus === "failed") {
    return {
      color: "text-destructive",
      bgColor: "bg-destructive/10",
      borderColor: "border-destructive/30",
      icon: XCircle,
      label: t("toolbox.ssl.connectionFailed"),
    }
  }

  if (result.connectionStatus === "http") {
    return {
      color: "text-yellow-600 dark:text-yellow-500",
      bgColor: "bg-yellow-500/10",
      borderColor: "border-yellow-500/30",
      icon: Unlock,
      label: t("toolbox.ssl.httpOnly"),
    }
  }

  if (!result.certInfo) {
    return {
      color: "text-muted-foreground",
      bgColor: "bg-muted/50",
      borderColor: "border-muted",
      icon: AlertTriangle,
      label: t("toolbox.ssl.noCertInfo"),
    }
  }

  const cert = result.certInfo

  if (!cert.isValid || cert.isExpired) {
    return {
      color: "text-destructive",
      bgColor: "bg-destructive/10",
      borderColor: "border-destructive/30",
      icon: XCircle,
      label: cert.isExpired ? t("toolbox.ssl.expired") : t("toolbox.ssl.invalid"),
    }
  }

  if (cert.daysRemaining <= 30) {
    return {
      color: "text-yellow-600 dark:text-yellow-500",
      bgColor: "bg-yellow-500/10",
      borderColor: "border-yellow-500/30",
      icon: AlertTriangle,
      label: t("toolbox.ssl.expiringSoon"),
    }
  }

  return {
    color: "text-green-600 dark:text-green-500",
    bgColor: "bg-green-500/10",
    borderColor: "border-green-500/30",
    icon: CheckCircle,
    label: t("toolbox.ssl.valid"),
  }
}

export function SslCheck() {
  const { t } = useTranslation()
  const [domain, setDomain] = useState("")
  const [port, setPort] = useState("")
  const [chainOpen, setChainOpen] = useState(false)

  const { isLoading, result, execute } = useToolboxQuery<SslCheckResult>()

  const handleLookup = () => {
    const trimmed = domain.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterDomain"))
      return
    }

    const portNum = port.trim() ? Number.parseInt(port.trim(), 10) : undefined
    if (
      port.trim() &&
      (Number.isNaN(portNum) || portNum! < NETWORK.MIN_PORT || portNum! > NETWORK.MAX_PORT)
    ) {
      toast.error(t("toolbox.ssl.invalidPort"))
      return
    }

    execute(() => toolboxService.sslCheck(trimmed, portNum), {
      type: "ssl",
      query: portNum ? `${trimmed}:${portNum}` : trimmed,
    })
  }

  const handleKeyDown = useEnterKeyHandler(handleLookup)

  const handleHistorySelect = (item: { query: string }) => {
    const parts = item.query.split(":")
    if (parts.length === 2 && /^\d+$/.test(parts[1])) {
      setDomain(parts[0])
      setPort(parts[1])
    } else {
      setDomain(item.query)
      setPort("")
    }
  }

  // 使用 useMemo 缓存状态信息计算
  const statusInfo = useMemo(() => getStatusInfo(result, t), [result, t])
  const cert = result?.certInfo

  return (
    <ToolCard title={t("toolbox.sslCheck")}>
      {/* 查询输入 - 域名 + 端口内嵌样式（与 DNS 查询一致） */}
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
          <div className="flex items-center border-l px-3">
            <span className="text-muted-foreground text-sm">:</span>
            <Input
              placeholder="443"
              value={port}
              onChange={(e) => setPort(e.target.value.replace(/\D/g, ""))}
              onKeyDown={handleKeyDown}
              disabled={isLoading}
              className="w-16 border-0 px-1 text-center shadow-none"
            />
          </div>
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

      <HistoryChips type="ssl" onSelect={handleHistorySelect} />

      {result && statusInfo && (
        <div className="space-y-4">
          {/* 连接状态卡片 */}
          <div className={`rounded-lg border p-4 ${statusInfo.bgColor} ${statusInfo.borderColor}`}>
            <div className="flex items-center gap-3">
              <statusInfo.icon className={`h-8 w-8 ${statusInfo.color}`} />
              <div className="flex-1">
                <div className={`font-semibold text-lg ${statusInfo.color}`}>
                  {statusInfo.label}
                </div>
                <div className="flex items-center gap-2 text-muted-foreground text-sm">
                  <Globe className="h-4 w-4" />
                  <span>{result.domain}</span>
                  <Badge variant="outline">:{result.port}</Badge>
                  <Badge
                    variant={
                      result.connectionStatus === "https"
                        ? "default"
                        : result.connectionStatus === "http"
                          ? "secondary"
                          : "destructive"
                    }
                  >
                    {result.connectionStatus.toUpperCase()}
                  </Badge>
                </div>
                {cert && (
                  <div className="mt-1 text-muted-foreground text-sm">
                    {cert.isExpired
                      ? t("toolbox.ssl.expiredDays", { count: Math.abs(cert.daysRemaining) })
                      : t("toolbox.ssl.daysRemaining", { count: cert.daysRemaining })}
                  </div>
                )}
                {result.error && (
                  <div className="mt-1 text-destructive text-sm">{result.error}</div>
                )}
              </div>
            </div>
          </div>

          {/* HTTP 连接警告 */}
          {result.connectionStatus === "http" && (
            <div className="rounded-lg border border-yellow-500/30 bg-yellow-500/10 p-4">
              <div className="flex items-start gap-3">
                <Unlock className="h-5 w-5 text-yellow-600 dark:text-yellow-500" />
                <div className="text-sm">
                  <div className="font-medium text-yellow-600 dark:text-yellow-500">
                    {t("toolbox.ssl.httpWarningTitle")}
                  </div>
                  <div className="text-muted-foreground">{t("toolbox.ssl.httpWarningDesc")}</div>
                </div>
              </div>
            </div>
          )}

          {/* 证书详情 */}
          {cert && (
            <>
              {/* 证书信息 */}
              <InfoCard icon={<Lock className="h-5 w-5" />} title={t("toolbox.ssl.certInfo")}>
                <div className="grid grid-cols-1 gap-3 text-sm">
                  <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:gap-2">
                    <span className="text-muted-foreground">{t("toolbox.ssl.subject")}:</span>
                    <span className="break-all font-mono">{cert.subject}</span>
                  </div>
                  <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:gap-2">
                    <span className="text-muted-foreground">{t("toolbox.ssl.issuer")}:</span>
                    <span className="break-all font-mono">{cert.issuer}</span>
                  </div>
                  <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:gap-2">
                    <span className="text-muted-foreground">{t("toolbox.ssl.serialNumber")}:</span>
                    <CopyableText value={cert.serialNumber} className="break-all font-mono">
                      {cert.serialNumber}
                    </CopyableText>
                  </div>
                  <div className="flex flex-col gap-1 sm:flex-row sm:items-center sm:gap-2">
                    <span className="text-muted-foreground">{t("toolbox.ssl.algorithm")}:</span>
                    <span className="font-mono">{cert.signatureAlgorithm}</span>
                  </div>
                </div>
              </InfoCard>

              {/* 有效期 */}
              <InfoCard icon={<Clock className="h-5 w-5" />} title={t("toolbox.ssl.validity")}>
                <div className="grid grid-cols-1 gap-3 text-sm sm:grid-cols-2">
                  <div>
                    <span className="text-muted-foreground">{t("toolbox.ssl.validFrom")}:</span>
                    <div className="font-mono">{cert.validFrom}</div>
                  </div>
                  <div>
                    <span className="text-muted-foreground">{t("toolbox.ssl.validTo")}:</span>
                    <div className="font-mono">{cert.validTo}</div>
                  </div>
                </div>
              </InfoCard>

              {/* SAN 列表 */}
              {cert.san.length > 0 && (
                <InfoCard
                  icon={<Link className="h-5 w-5" />}
                  title={`${t("toolbox.ssl.san")} (${cert.san.length})`}
                >
                  <div className="flex flex-wrap gap-2">
                    {cert.san.map((name) => (
                      <CopyableText
                        key={name}
                        value={name}
                        className="rounded-full bg-primary/10 px-3 py-1 font-mono text-primary text-xs hover:bg-primary/20"
                      >
                        {name}
                      </CopyableText>
                    ))}
                  </div>
                </InfoCard>
              )}

              {/* 证书链 */}
              {cert.certificateChain.length > 0 && (
                <Collapsible open={chainOpen} onOpenChange={setChainOpen}>
                  <div className="rounded-lg border bg-card">
                    <CollapsibleTrigger className="flex w-full items-center justify-between p-4 hover:bg-muted/50">
                      <div className="flex items-center gap-2">
                        <Shield className="h-5 w-5 text-primary" />
                        <span className="font-medium">
                          {t("toolbox.ssl.chain")} ({cert.certificateChain.length})
                        </span>
                      </div>
                      <ChevronDown
                        className={`h-4 w-4 transition-transform ${chainOpen ? "rotate-180" : ""}`}
                      />
                    </CollapsibleTrigger>
                    <CollapsibleContent>
                      <div className="border-t p-4">
                        <div className="space-y-3">
                          {cert.certificateChain.map((chainCert, index) => (
                            <div
                              key={`cert-${index}-${chainCert.subject}`}
                              className="rounded border p-3 text-sm"
                            >
                              <div className="mb-1 flex items-center gap-2">
                                <span className="rounded bg-muted px-2 py-0.5 text-muted-foreground text-xs">
                                  {index + 1}
                                </span>
                                {chainCert.isCa && (
                                  <span className="rounded bg-blue-500/10 px-2 py-0.5 text-blue-600 text-xs dark:text-blue-400">
                                    CA
                                  </span>
                                )}
                              </div>
                              <div className="text-muted-foreground">
                                <span className="text-foreground">{t("toolbox.ssl.subject")}:</span>{" "}
                                {chainCert.subject}
                              </div>
                              <div className="text-muted-foreground">
                                <span className="text-foreground">{t("toolbox.ssl.issuer")}:</span>{" "}
                                {chainCert.issuer}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    </CollapsibleContent>
                  </div>
                </Collapsible>
              )}
            </>
          )}
        </div>
      )}
    </ToolCard>
  )
}
