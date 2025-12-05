import { DnsRecordTable } from "@/components/dns/DnsRecordTable"
import { useAccountStore, useDomainStore } from "@/stores"
import { Globe } from "lucide-react"
import { useMemo } from "react"
import { useTranslation } from "react-i18next"

export function MainContent() {
  const { t } = useTranslation()
  const { selectedAccountId, accounts, providers } = useAccountStore()
  const { selectedDomainId, domains } = useDomainStore()

  const selectedDomain = domains.find((d) => d.id === selectedDomainId)
  const selectedAccount = accounts.find((a) => a.id === selectedAccountId)

  // 获取当前账户对应的提供商功能
  const providerFeatures = useMemo(() => {
    if (!selectedAccount) return null
    const provider = providers.find((p) => p.id === selectedAccount.provider)
    return provider?.features ?? null
  }, [selectedAccount, providers])

  if (!(selectedDomainId && selectedAccountId)) {
    return (
      <main className="flex flex-1 items-center justify-center bg-muted/30">
        <div className="text-center text-muted-foreground">
          <Globe className="mx-auto mb-4 h-12 w-12 opacity-50" />
          <p className="text-lg">{t("main.selectDomain")}</p>
          <p className="mt-1 text-sm">{t("main.selectDomainHint")}</p>
        </div>
      </main>
    )
  }

  return (
    <main className="flex flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="border-b bg-background px-6 py-4">
        <h2 className="font-semibold text-xl">{selectedDomain?.name}</h2>
        <p className="mt-1 text-muted-foreground text-sm">
          {t("main.manageDns", { domain: selectedDomain?.name })}
        </p>
      </div>

      {/* Content */}
      <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <DnsRecordTable
          accountId={selectedAccountId}
          domainId={selectedDomainId}
          supportsProxy={providerFeatures?.proxy ?? false}
        />
      </div>
    </main>
  )
}
