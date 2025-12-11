import { ArrowLeft } from "lucide-react"
import { useEffect, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate, useParams } from "react-router-dom"
import { DnsRecordTable } from "@/components/dns/DnsRecordTable"
import { addRecentDomain } from "@/components/home/HomePage"
import { Button } from "@/components/ui/button"
import { useAccountStore, useDomainStore } from "@/stores"

export function DnsRecordPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { accountId, domainId } = useParams<{ accountId: string; domainId: string }>()
  const { accounts, providers } = useAccountStore()
  const { getDomainsForAccount } = useDomainStore()

  const selectedAccount = accounts.find((a) => a.id === accountId)

  // 从缓存中获取域名信息
  const selectedDomain = useMemo(() => {
    if (!accountId) return undefined
    const domains = getDomainsForAccount(accountId)
    return domains.find((d) => d.id === domainId)
  }, [getDomainsForAccount, accountId, domainId])

  // 添加到最近访问记录
  useEffect(() => {
    if (selectedAccount && selectedDomain && accountId && domainId) {
      addRecentDomain({
        accountId,
        domainId,
        domainName: selectedDomain.name,
        accountName: selectedAccount.name,
        provider: selectedAccount.provider,
      })
    }
  }, [selectedAccount, selectedDomain, accountId, domainId])

  // 如果参数缺失，重定向到域名列表
  useEffect(() => {
    if (!(accountId && domainId)) {
      navigate("/domains", { replace: true })
    }
  }, [accountId, domainId, navigate])

  // 参数缺失时不渲染
  if (!(accountId && domainId)) {
    return null
  }

  // 获取当前账户对应的提供商功能
  const providerFeatures = useMemo(() => {
    if (!selectedAccount) return null
    const provider = providers.find((p) => p.id === selectedAccount.provider)
    return provider?.features ?? null
  }, [selectedAccount, providers])

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 border-b bg-background px-4 py-3 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={() => navigate("/domains")}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="min-w-0 flex-1">
          <h2 className="truncate font-semibold text-xl">
            {selectedDomain?.name || t("common.loading")}
          </h2>
          <p className="truncate text-muted-foreground text-sm">
            {t("dns.title")} · {selectedAccount?.name}
          </p>
        </div>
      </div>

      {/* DNS 记录表格 */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DnsRecordTable
          accountId={accountId}
          domainId={domainId}
          supportsProxy={providerFeatures?.proxy ?? false}
        />
      </div>
    </div>
  )
}
