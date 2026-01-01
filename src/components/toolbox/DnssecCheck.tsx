import {
  AlertCircle,
  Key,
  Loader2,
  Search,
  Shield,
  ShieldAlert,
  ShieldCheck,
  XCircle,
} from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
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
import type { DnssecResult } from "@/types"
import { HistoryChips } from "./HistoryChips"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { CopyableText, ToolCard } from "./shared"

export function DnssecCheck() {
  const { t } = useTranslation()
  const isMobile = useIsMobile()
  const [domain, setDomain] = useState("")
  const [nameserver, setNameserver] = useState("")

  const { isLoading, result, execute } = useToolboxQuery<DnssecResult>()

  const handleCheck = async () => {
    const trimmed = domain.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterDomain"))
      return
    }

    const ns = nameserver.trim() || null
    await execute(() => toolboxService.dnssecCheck(trimmed, ns), {
      type: "dnssec",
      query: trimmed,
    })
  }

  const handleKeyDown = useEnterKeyHandler(handleCheck)

  // 获取验证状态徽章
  const getValidationBadge = (status: string) => {
    switch (status) {
      case "secure":
        return (
          <Badge className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100">
            <ShieldCheck className="mr-1 h-3 w-3" />
            {t("toolbox.dnssec.status.secure")}
          </Badge>
        )
      case "insecure":
        return (
          <Badge className="bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-100">
            <ShieldAlert className="mr-1 h-3 w-3" />
            {t("toolbox.dnssec.status.insecure")}
          </Badge>
        )
      case "bogus":
        return (
          <Badge className="bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-100">
            <XCircle className="mr-1 h-3 w-3" />
            {t("toolbox.dnssec.status.bogus")}
          </Badge>
        )
      default:
        return (
          <Badge className="bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100">
            <AlertCircle className="mr-1 h-3 w-3" />
            {t("toolbox.dnssec.status.indeterminate")}
          </Badge>
        )
    }
  }

  return (
    <ToolCard title={t("toolbox.dnssec.title")}>
      {/* 查询输入 */}
      <div className="flex flex-col gap-2 sm:flex-row">
        <Input
          placeholder={t("toolbox.dnssec.domainPlaceholder")}
          value={domain}
          onChange={(e) => setDomain(e.target.value)}
          onKeyDown={handleKeyDown}
          className="flex-1"
        />
        <Input
          placeholder={t("toolbox.dnssec.nameserverPlaceholder")}
          value={nameserver}
          onChange={(e) => setNameserver(e.target.value)}
          onKeyDown={handleKeyDown}
          className="flex-1"
        />
        <Button onClick={handleCheck} disabled={isLoading} className="w-full sm:w-auto">
          {isLoading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Search className="mr-2 h-4 w-4" />
          )}
          {t("toolbox.check")}
        </Button>
      </div>

      {/* 历史记录 */}
      <HistoryChips
        type="dnssec"
        onSelect={(item) => {
          setDomain(item.query)
        }}
      />

      {/* 结果展示 */}
      {result && (
        <div className="space-y-4">
          {/* 概览卡片 */}
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div className="rounded-lg border bg-card p-4 text-card-foreground">
              <div className="flex items-center gap-2">
                <Shield className="h-5 w-5 text-muted-foreground" />
                <div className="text-muted-foreground text-sm">
                  {t("toolbox.dnssec.dnssecEnabled")}
                </div>
              </div>
              <div className="mt-2 font-bold text-2xl">
                {result.dnssecEnabled ? (
                  <span className="text-green-600">{t("common.yes")}</span>
                ) : (
                  <span className="text-red-600">{t("common.no")}</span>
                )}
              </div>
            </div>

            <div className="rounded-lg border bg-card p-4 text-card-foreground">
              <div className="flex items-center gap-2">
                <Key className="h-5 w-5 text-muted-foreground" />
                <div className="text-muted-foreground text-sm">
                  {t("toolbox.dnssec.validationStatus")}
                </div>
              </div>
              <div className="mt-2">{getValidationBadge(result.validationStatus)}</div>
            </div>

            <div className="rounded-lg border bg-card p-4 text-card-foreground">
              <div className="flex items-center gap-2">
                <AlertCircle className="h-5 w-5 text-muted-foreground" />
                <div className="text-muted-foreground text-sm">
                  {t("toolbox.dnssec.responseTime")}
                </div>
              </div>
              <div className="mt-2 font-bold text-2xl">{result.responseTimeMs}ms</div>
            </div>
          </div>

          {/* DNSKEY 记录 */}
          {result.dnskeyRecords.length > 0 && (
            <div className="space-y-2">
              <h3 className="font-semibold">{t("toolbox.dnssec.dnskeyRecords")}</h3>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("toolbox.dnssec.keyType")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.flags")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.algorithm")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.keyTag")}</TableHead>
                      {!isMobile && <TableHead>{t("toolbox.dnssec.publicKey")}</TableHead>}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {result.dnskeyRecords.map((record) => (
                      <TableRow key={record.keyTag}>
                        <TableCell>
                          <Badge variant="outline">{record.keyType}</Badge>
                        </TableCell>
                        <TableCell>{record.flags}</TableCell>
                        <TableCell className="max-w-xs">
                          <div className="truncate" title={record.algorithmName}>
                            {record.algorithmName}
                          </div>
                        </TableCell>
                        <TableCell>{record.keyTag}</TableCell>
                        {!isMobile && (
                          <TableCell className="max-w-xs">
                            <CopyableText
                              value={record.publicKey}
                              className="block truncate font-mono text-sm"
                            />
                          </TableCell>
                        )}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}

          {/* DS 记录 */}
          {result.dsRecords.length > 0 && (
            <div className="space-y-2">
              <h3 className="font-semibold">{t("toolbox.dnssec.dsRecords")}</h3>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("toolbox.dnssec.keyTag")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.algorithm")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.digestType")}</TableHead>
                      {!isMobile && <TableHead>{t("toolbox.dnssec.digest")}</TableHead>}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {result.dsRecords.map((record) => (
                      <TableRow key={record.keyTag}>
                        <TableCell>{record.keyTag}</TableCell>
                        <TableCell className="max-w-xs">
                          <div className="truncate" title={record.algorithmName}>
                            {record.algorithmName}
                          </div>
                        </TableCell>
                        <TableCell className="max-w-xs">
                          <div className="truncate" title={record.digestTypeName}>
                            {record.digestTypeName}
                          </div>
                        </TableCell>
                        {!isMobile && (
                          <TableCell className="max-w-xs">
                            <CopyableText
                              value={record.digest}
                              className="block truncate font-mono text-sm"
                            />
                          </TableCell>
                        )}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}

          {/* RRSIG 记录 */}
          {result.rrsigRecords.length > 0 && (
            <div className="space-y-2">
              <h3 className="font-semibold">{t("toolbox.dnssec.rrsigRecords")}</h3>
              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>{t("toolbox.dnssec.typeCovered")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.algorithm")}</TableHead>
                      <TableHead>{t("toolbox.dnssec.keyTag")}</TableHead>
                      {!isMobile && <TableHead>{t("toolbox.dnssec.signerName")}</TableHead>}
                      {!isMobile && (
                        <TableHead>{t("toolbox.dnssec.signatureExpiration")}</TableHead>
                      )}
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {result.rrsigRecords.map((record, idx) => (
                      <TableRow key={`${record.typeCovered}-${record.keyTag}-${idx}`}>
                        <TableCell>
                          <Badge variant="outline">{record.typeCovered}</Badge>
                        </TableCell>
                        <TableCell className="max-w-xs">
                          <div className="truncate" title={record.algorithmName}>
                            {record.algorithmName}
                          </div>
                        </TableCell>
                        <TableCell>{record.keyTag}</TableCell>
                        {!isMobile && (
                          <TableCell className="max-w-xs">
                            <div className="truncate" title={record.signerName}>
                              {record.signerName}
                            </div>
                          </TableCell>
                        )}
                        {!isMobile && <TableCell>{record.signatureExpiration}</TableCell>}
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>
            </div>
          )}

          {/* 没有 DNSSEC 记录时的提示 */}
          {!result.dnssecEnabled && (
            <div className="rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-800 dark:bg-yellow-950">
              <div className="flex items-start gap-3">
                <AlertCircle className="mt-0.5 h-5 w-5 flex-shrink-0 text-yellow-600 dark:text-yellow-400" />
                <div className="text-sm text-yellow-800 dark:text-yellow-200">
                  {t("toolbox.dnssec.notEnabled")}
                </div>
              </div>
            </div>
          )}

          {/* DNS 服务器信息 */}
          <div className="text-muted-foreground text-sm">
            {t("toolbox.dnssec.nameserverUsed")}: {result.nameserver}
          </div>
        </div>
      )}
    </ToolCard>
  )
}
