import { ArrowLeft, StickyNote } from "lucide-react"
import { useEffect, useMemo } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate, useParams } from "react-router-dom"
import { useShallow } from "zustand/react/shallow"
import { DnsRecordTable } from "@/components/dns/DnsRecordTable"
import { Button } from "@/components/ui/button"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { addRecentDomain } from "@/lib/recent-domains"
import { useAccountStore, useDomainStore } from "@/stores"

export function DnsRecordPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { accountId, domainId } = useParams<{ accountId: string; domainId: string }>()

  // 使用 useShallow 优化 store 订阅粒度
  const { accounts, providers } = useAccountStore(
    useShallow((state) => ({
      accounts: state.accounts,
      providers: state.providers,
    }))
  )
  const getDomainsForAccount = useDomainStore((state) => state.getDomainsForAccount)

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

  // 获取当前账户对应的提供商功能（必须在早期 return 之前调用）
  const providerFeatures = useMemo(() => {
    if (!selectedAccount) return null
    const provider = providers.find((p) => p.id === selectedAccount.provider)
    return provider?.features ?? null
  }, [selectedAccount, providers])

  // 参数缺失时不渲染
  if (!(accountId && domainId)) {
    return null
  }

  return (
    <PageLayout>
      <PageHeader
        title={selectedDomain?.name || t("common.loading")}
        subtitle={`${t("dns.title")} · ${selectedAccount?.name}`}
        showMobileMenu={false}
        backButton={
          <Button variant="ghost" size="icon" onClick={() => navigate(-1)}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
        }
        actions={
          selectedDomain?.metadata?.note ? (
            <div className="hidden max-w-md items-center gap-2 rounded-md border bg-muted/50 px-3 py-1.5 md:flex">
              <StickyNote className="h-4 w-4 shrink-0 text-muted-foreground" />
              <span className="truncate text-muted-foreground text-sm">
                {selectedDomain.metadata.note}
              </span>
            </div>
          ) : undefined
        }
      />

      {/* DNS 记录表格 */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DnsRecordTable
          accountId={accountId}
          domainId={domainId}
          supportsProxy={providerFeatures?.proxy ?? false}
        />
      </div>
    </PageLayout>
  )
}
