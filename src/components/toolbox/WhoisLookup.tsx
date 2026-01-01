import { ChevronDown, ChevronUp } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import type { WhoisResult } from "@/types"
import { toolboxService, useToolboxQuery } from "./hooks/useToolboxQuery"
import { QueryInput, ToolCard } from "./shared"

export function WhoisLookup() {
  const { t } = useTranslation()
  const [domain, setDomain] = useState("")
  const [showRaw, setShowRaw] = useState(false)

  const { isLoading, result, execute } = useToolboxQuery<WhoisResult>()

  const handleLookup = () => {
    const trimmed = domain.trim()
    if (!trimmed) {
      toast.error(t("toolbox.enterDomain"))
      return
    }
    execute(() => toolboxService.whoisLookup(trimmed), {
      type: "whois",
      query: trimmed,
    })
  }

  return (
    <ToolCard title={t("toolbox.whoisLookup")}>
      <QueryInput
        value={domain}
        onChange={setDomain}
        onSubmit={handleLookup}
        isLoading={isLoading}
        placeholder={t("toolbox.domainPlaceholder")}
        historyType="whois"
        onHistorySelect={(item) => setDomain(item.query)}
      />

      {result && (
        <div className="space-y-3 pt-2">
          <div className="grid grid-cols-1 gap-3 text-sm sm:grid-cols-2 sm:gap-4">
            <div>
              <span className="text-muted-foreground">{t("toolbox.whois.domain")}:</span>
              <span className="ml-2 font-medium">{result.domain}</span>
            </div>
            {result.registrar && (
              <div>
                <span className="text-muted-foreground">{t("toolbox.whois.registrar")}:</span>
                <span className="ml-2">{result.registrar}</span>
              </div>
            )}
            {result.creationDate && (
              <div>
                <span className="text-muted-foreground">{t("toolbox.whois.creationDate")}:</span>
                <span className="ml-2">{result.creationDate}</span>
              </div>
            )}
            {result.expirationDate && (
              <div>
                <span className="text-muted-foreground">{t("toolbox.whois.expirationDate")}:</span>
                <span className="ml-2">{result.expirationDate}</span>
              </div>
            )}
            {result.updatedDate && (
              <div>
                <span className="text-muted-foreground">{t("toolbox.whois.updatedDate")}:</span>
                <span className="ml-2">{result.updatedDate}</span>
              </div>
            )}
          </div>

          {result.nameServers.length > 0 && (
            <div className="text-sm">
              <span className="text-muted-foreground">{t("toolbox.whois.nameServers")}:</span>
              <div className="mt-1 flex flex-wrap gap-1">
                {result.nameServers.map((ns) => (
                  <span key={ns} className="rounded bg-muted px-2 py-0.5 font-mono text-xs">
                    {ns}
                  </span>
                ))}
              </div>
            </div>
          )}

          {result.status.length > 0 && (
            <div className="text-sm">
              <span className="text-muted-foreground">{t("toolbox.whois.status")}:</span>
              <div className="mt-1 flex flex-wrap gap-1">
                {result.status.map((s) => (
                  <span key={s} className="rounded bg-muted px-2 py-0.5 text-xs">
                    {s}
                  </span>
                ))}
              </div>
            </div>
          )}

          {result.raw && (
            <div className="text-sm">
              <Button
                variant="ghost"
                size="sm"
                className="h-auto px-0 text-muted-foreground hover:text-foreground"
                onClick={() => setShowRaw(!showRaw)}
              >
                {showRaw ? (
                  <ChevronUp className="mr-1 h-4 w-4" />
                ) : (
                  <ChevronDown className="mr-1 h-4 w-4" />
                )}
                {t("toolbox.whois.rawData")}
              </Button>
              {showRaw && (
                <pre className="mt-2 max-h-[300px] overflow-y-auto whitespace-pre-wrap break-all rounded bg-muted p-3 font-mono text-xs">
                  {result.raw}
                </pre>
              )}
            </div>
          )}
        </div>
      )}
    </ToolCard>
  )
}
