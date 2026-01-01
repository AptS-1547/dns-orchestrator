import { Globe, MapPin, Server } from "lucide-react"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Badge } from "@/components/ui/badge"
import type { IpGeoInfo, IpLookupResult } from "@/types"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { CopyableText, InfoCard, QueryInput, ToolCard } from "./shared"

/** 国家代码转国旗 emoji */
function countryCodeToFlag(countryCode: string): string {
  const codePoints = countryCode
    .toUpperCase()
    .split("")
    .map((char) => 127397 + char.charCodeAt(0))
  return String.fromCodePoint(...codePoints)
}

export function IpLookup() {
  const { t } = useTranslation()
  const [query, setQuery] = useState("")

  const { isLoading, result, execute } = useToolboxQuery<IpLookupResult>()

  const handleLookup = () => {
    const trimmed = query.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterIpOrDomain"))
      return
    }
    execute(() => toolboxService.ipLookup(trimmed), {
      type: "ip",
      query: trimmed,
    })
  }

  /** 渲染单个 IP 结果 - 使用 useCallback 缓存 */
  const renderIpResult = useCallback(
    (ipInfo: IpGeoInfo) => (
      <div key={ipInfo.ip} className="space-y-4">
        {/* IP 标题（多结果时显示） */}
        {result && result.results.length > 1 && (
          <div className="flex items-center gap-2">
            <Badge variant={ipInfo.ipVersion === "IPv6" ? "secondary" : "default"}>
              {ipInfo.ipVersion}
            </Badge>
            <CopyableText value={ipInfo.ip} className="font-mono text-sm">
              {ipInfo.ip}
            </CopyableText>
          </div>
        )}

        {/* 位置信息卡片 */}
        <InfoCard icon={<MapPin className="h-5 w-5" />} title={t("toolbox.ip.location")}>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {ipInfo.country && (
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-sm">{t("toolbox.ip.country")}:</span>
                <span className="font-medium">
                  {ipInfo.countryCode && countryCodeToFlag(ipInfo.countryCode)} {ipInfo.country}
                </span>
              </div>
            )}
            {ipInfo.region && (
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-sm">{t("toolbox.ip.region")}:</span>
                <span className="font-medium">{ipInfo.region}</span>
              </div>
            )}
            {ipInfo.city && (
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-sm">{t("toolbox.ip.city")}:</span>
                <span className="font-medium">{ipInfo.city}</span>
              </div>
            )}
            {ipInfo.timezone && (
              <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-sm">{t("toolbox.ip.timezone")}:</span>
                <span className="font-medium">{ipInfo.timezone}</span>
              </div>
            )}
            {ipInfo.latitude != null && ipInfo.longitude != null && (
              <div className="flex items-center gap-2 sm:col-span-2">
                <span className="text-muted-foreground text-sm">
                  {t("toolbox.ip.coordinates")}:
                </span>
                <span className="font-mono text-sm">
                  {ipInfo.latitude.toFixed(4)}, {ipInfo.longitude.toFixed(4)}
                </span>
              </div>
            )}
          </div>
        </InfoCard>

        {/* 网络信息卡片 */}
        <InfoCard icon={<Server className="h-5 w-5" />} title={t("toolbox.ip.network")}>
          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            {/* 单结果时显示 IP - 独占整行 */}
            {result && result.results.length === 1 && (
              <div className="sm:col-span-2">
                <div className="flex items-center gap-2 text-muted-foreground text-sm">
                  <span>{t("toolbox.ip.ip")}:</span>
                  <Badge variant={ipInfo.ipVersion === "IPv6" ? "secondary" : "outline"}>
                    {ipInfo.ipVersion}
                  </Badge>
                </div>
                <CopyableText value={ipInfo.ip} className="break-all font-mono text-sm">
                  {ipInfo.ip}
                </CopyableText>
              </div>
            )}
            {ipInfo.isp && (
              <div className="sm:col-span-2">
                <div className="text-muted-foreground text-sm">{t("toolbox.ip.isp")}:</div>
                <div className="font-medium">{ipInfo.isp}</div>
              </div>
            )}
            {ipInfo.org && (
              <div className="sm:col-span-2">
                <div className="text-muted-foreground text-sm">{t("toolbox.ip.org")}:</div>
                <div className="font-medium">{ipInfo.org}</div>
              </div>
            )}
            {ipInfo.asn && (
              <div className="flex items-center gap-2 sm:col-span-2">
                <Globe className="h-4 w-4 text-muted-foreground" />
                <span className="text-muted-foreground text-sm">{t("toolbox.ip.asn")}:</span>
                <span className="font-mono text-sm">{ipInfo.asn}</span>
                {ipInfo.asName && (
                  <span className="text-muted-foreground text-sm">({ipInfo.asName})</span>
                )}
              </div>
            )}
          </div>
        </InfoCard>
      </div>
    ),
    [result, t]
  )

  return (
    <ToolCard title={t("toolbox.ipLookup")}>
      <QueryInput
        value={query}
        onChange={setQuery}
        onSubmit={handleLookup}
        isLoading={isLoading}
        placeholder={t("toolbox.ipOrDomainPlaceholder")}
        historyType="ip"
        onHistorySelect={(item) => setQuery(item.query)}
      />

      {result && (
        <div className="space-y-6">
          {/* 域名查询时显示解析的域名 */}
          {result.isDomain && (
            <div className="flex items-center gap-2 text-muted-foreground text-sm">
              <span>{t("toolbox.ip.resolvedFrom")}:</span>
              <span className="font-mono">{result.query}</span>
              <Badge variant="outline">{result.results.length} IP(s)</Badge>
            </div>
          )}

          {/* 渲染每个 IP 结果 */}
          {result.results.map((ipInfo) => renderIpResult(ipInfo))}
        </div>
      )}
    </ToolCard>
  )
}
