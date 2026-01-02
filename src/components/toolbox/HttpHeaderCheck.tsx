import { Copy, FileCode, Info, Loader2, Plus, Send, Shield, Trash2 } from "lucide-react"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Textarea } from "@/components/ui/textarea"
import { useCopyToClipboard } from "@/hooks/useCopyToClipboard"
import { toolboxService } from "@/services/toolbox.service"
import type {
  HttpHeader,
  HttpHeaderCheckRequest,
  HttpHeaderCheckResult,
  HttpMethod,
  SecurityHeaderAnalysis,
} from "@/types"
import { HeaderItem } from "./HeaderItem"
import { HistoryChips } from "./HistoryChips"
import { useToolboxQuery } from "./hooks/useToolboxQuery"

const HTTP_METHODS: HttpMethod[] = ["GET", "HEAD", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"]

interface CustomHeaderWithId extends HttpHeader {
  id: string
}

/** 获取安全头状态徽章样式 */
function getStatusBadgeVariant(
  status: SecurityHeaderAnalysis["status"]
): "default" | "secondary" | "destructive" {
  switch (status) {
    case "good":
      return "default"
    case "warning":
      return "secondary"
    case "missing":
      return "destructive"
    default:
      return "default"
  }
}

export function HttpHeaderCheck() {
  const { t } = useTranslation()

  // Form state
  const [url, setUrl] = useState("")
  const [method, setMethod] = useState<HttpMethod>("GET")
  const [customHeaders, setCustomHeaders] = useState<CustomHeaderWithId[]>([])
  const [body, setBody] = useState("")
  const [contentType, setContentType] = useState("application/json")

  // Query Hook
  const { isLoading, result, execute } = useToolboxQuery<HttpHeaderCheckResult>()

  // UI state
  const [expandedHeaders, setExpandedHeaders] = useState<Set<number>>(new Set())

  // Copy hook
  const copyToClipboard = useCopyToClipboard()

  const handleAddHeader = useCallback(() => {
    setCustomHeaders((prev) => [
      ...prev,
      {
        id: crypto.randomUUID(),
        name: "",
        value: "",
      },
    ])
  }, [])

  const handleRemoveHeader = useCallback((id: string) => {
    setCustomHeaders((prev) => prev.filter((h) => h.id !== id))
  }, [])

  const handleHeaderChange = useCallback((id: string, field: "name" | "value", value: string) => {
    setCustomHeaders((prev) => prev.map((h) => (h.id === id ? { ...h, [field]: value } : h)))
  }, [])

  const handleQuery = useCallback(async () => {
    if (!url) {
      toast.error(t("toolbox.httpHeaderCheck.urlRequired"))
      return
    }

    const request: HttpHeaderCheckRequest = {
      url,
      method,
      customHeaders: customHeaders.filter((h) => h.name && h.value),
      body: ["POST", "PUT", "PATCH"].includes(method) && body ? body : undefined,
      contentType: ["POST", "PUT", "PATCH"].includes(method) && body ? contentType : undefined,
    }

    await execute(() => toolboxService.httpHeaderCheck(request), { type: "http", query: url })
  }, [url, method, customHeaders, body, contentType, execute, t])

  return (
    <div className="space-y-6">
      {/* 查询表单 */}
      <Card>
        <CardHeader>
          <CardTitle>{t("toolbox.httpHeaderCheck.title")}</CardTitle>
          <CardDescription>{t("toolbox.httpHeaderCheck.description")}</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* URL 和 Method */}
          <div className="grid grid-cols-1 gap-4 md:grid-cols-5">
            <div className="md:col-span-4">
              <Label htmlFor="url">{t("toolbox.httpHeaderCheck.url")}</Label>
              <Input
                id="url"
                type="url"
                placeholder="https://example.com"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleQuery()}
              />
            </div>
            <div>
              <Label htmlFor="method">{t("toolbox.httpHeaderCheck.method")}</Label>
              <Select value={method} onValueChange={(v) => setMethod(v as HttpMethod)}>
                <SelectTrigger id="method">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {HTTP_METHODS.map((m) => (
                    <SelectItem key={m} value={m}>
                      {m}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>

          {/* 历史记录 */}
          <HistoryChips type="http" onSelect={(item) => setUrl(item.query)} />

          {/* 自定义请求头 */}
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>{t("toolbox.httpHeaderCheck.customHeaders")}</Label>
              <Button type="button" variant="outline" size="sm" onClick={handleAddHeader}>
                <Plus className="mr-1 size-4" />
                {t("toolbox.httpHeaderCheck.addHeader")}
              </Button>
            </div>
            {customHeaders.length > 0 && (
              <div className="space-y-2">
                {customHeaders.map((header) => (
                  <div key={header.id} className="grid grid-cols-1 gap-2 md:grid-cols-12">
                    <Input
                      className="md:col-span-5"
                      placeholder={t("toolbox.httpHeaderCheck.headerName")}
                      value={header.name}
                      onChange={(e) => handleHeaderChange(header.id, "name", e.target.value)}
                    />
                    <Input
                      className="md:col-span-6"
                      placeholder={t("toolbox.httpHeaderCheck.headerValue")}
                      value={header.value}
                      onChange={(e) => handleHeaderChange(header.id, "value", e.target.value)}
                    />
                    <Button
                      className="md:col-span-1"
                      type="button"
                      variant="ghost"
                      size="icon"
                      onClick={() => handleRemoveHeader(header.id)}
                    >
                      <Trash2 className="size-4" />
                    </Button>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* 请求体（仅 POST/PUT/PATCH） */}
          {["POST", "PUT", "PATCH"].includes(method) && (
            <div className="space-y-2">
              <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                <div>
                  <Label htmlFor="contentType">{t("toolbox.httpHeaderCheck.contentType")}</Label>
                  <Input
                    id="contentType"
                    value={contentType}
                    onChange={(e) => setContentType(e.target.value)}
                    placeholder="application/json"
                  />
                </div>
              </div>
              <div>
                <Label htmlFor="body">{t("toolbox.httpHeaderCheck.requestBody")}</Label>
                <Textarea
                  id="body"
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  placeholder={t("toolbox.httpHeaderCheck.requestBodyPlaceholder")}
                  className="min-h-[100px] font-mono text-sm"
                />
              </div>
            </div>
          )}

          <Button onClick={handleQuery} disabled={isLoading || !url} className="w-full">
            {isLoading ? (
              <>
                <Loader2 className="mr-2 size-4 animate-spin" />
                {t("common.loading")}
              </>
            ) : (
              <>
                <Send className="mr-2 size-4" />
                {t("toolbox.httpHeaderCheck.check")}
              </>
            )}
          </Button>
        </CardContent>
      </Card>

      {/* 查询结果 */}
      {result && (
        <div className="space-y-4">
          {/* 响应状态 */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">
                {t("toolbox.httpHeaderCheck.responseStatus")}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                <div>
                  <div className="text-muted-foreground text-sm">
                    {t("toolbox.httpHeaderCheck.statusCode")}
                  </div>
                  <div className="font-bold text-2xl">
                    <Badge variant={result.statusCode < 400 ? "default" : "destructive"}>
                      {result.statusCode} {result.statusText}
                    </Badge>
                  </div>
                </div>
                <div>
                  <div className="text-muted-foreground text-sm">
                    {t("toolbox.httpHeaderCheck.responseTime")}
                  </div>
                  <div className="font-bold text-2xl">{result.responseTimeMs} ms</div>
                </div>
                {result.contentLength !== undefined && (
                  <div>
                    <div className="text-muted-foreground text-sm">Content-Length</div>
                    <div className="font-bold text-2xl">
                      {(result.contentLength / 1024).toFixed(2)} KB
                    </div>
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          {/* 安全头分析 */}
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <Shield className="size-5" />
                <CardTitle className="text-lg">
                  {t("toolbox.httpHeaderCheck.securityAnalysis")}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {result.securityAnalysis.map((analysis) => (
                  <div
                    key={analysis.name}
                    className="flex items-start justify-between rounded-md border p-3"
                  >
                    <div className="flex-1 space-y-1">
                      <div className="flex items-center gap-2">
                        <code className="font-mono text-sm">{analysis.name}</code>
                        <Badge variant={getStatusBadgeVariant(analysis.status)}>
                          {t(`toolbox.httpHeaderCheck.status.${analysis.status}`)}
                        </Badge>
                      </div>
                      {analysis.recommendation && (
                        <div className="mt-2 flex items-start gap-2 text-amber-600 text-sm dark:text-amber-400">
                          <Info className="mt-0.5 size-4 shrink-0" />
                          <span>{analysis.recommendation}</span>
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* 所有响应头 */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">
                {t("toolbox.httpHeaderCheck.allHeaders")} ({result.headers.length})
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="max-h-[400px] space-y-2 overflow-y-auto">
                {result.headers.map((header, index) => (
                  <HeaderItem
                    key={`header-${header.name}-${index}`}
                    header={header}
                    index={index}
                    isExpanded={expandedHeaders.has(index)}
                    onToggle={(isOpen) => {
                      const newSet = new Set(expandedHeaders)
                      if (isOpen) {
                        newSet.add(index)
                      } else {
                        newSet.delete(index)
                      }
                      setExpandedHeaders(newSet)
                    }}
                    onCopy={() => copyToClipboard(`${header.name}: ${header.value}`)}
                  />
                ))}
              </div>
            </CardContent>
          </Card>

          {/* 原始报文 */}
          <Card>
            <CardHeader>
              <div className="flex items-center gap-2">
                <FileCode className="size-5" />
                <CardTitle className="text-lg">
                  {t("toolbox.httpHeaderCheck.rawMessages")}
                </CardTitle>
              </div>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* 原始请求 */}
              <div>
                <div className="mb-2 flex items-center justify-between">
                  <Label className="font-semibold text-sm">
                    {t("toolbox.httpHeaderCheck.rawRequest")}
                  </Label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyToClipboard(result.rawRequest)}
                  >
                    <Copy className="mr-1 size-4" />
                    {t("common.copy")}
                  </Button>
                </div>
                <pre className="max-w-full overflow-x-auto whitespace-pre-wrap break-all rounded-md bg-muted p-3 font-mono text-xs">
                  {result.rawRequest}
                </pre>
              </div>

              {/* 原始响应 */}
              <div>
                <div className="mb-2 flex items-center justify-between">
                  <Label className="font-semibold text-sm">
                    {t("toolbox.httpHeaderCheck.rawResponse")}
                  </Label>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => copyToClipboard(result.rawResponse)}
                  >
                    <Copy className="mr-1 size-4" />
                    {t("common.copy")}
                  </Button>
                </div>
                <pre className="max-h-[300px] max-w-full overflow-x-auto overflow-y-auto whitespace-pre-wrap break-all rounded-md bg-muted p-3 font-mono text-xs">
                  {result.rawResponse}
                </pre>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
