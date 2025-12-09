import { ChevronRight, Loader2, MoreHorizontal, Trash2, TriangleAlert } from "lucide-react"
import { useTranslation } from "react-i18next"
import { getProviderName, ProviderIcon } from "@/components/account/ProviderIcon"
import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import type { Account, Domain } from "@/types"
import { DomainTreeItem } from "./DomainTreeItem"

interface AccountTreeItemProps {
  account: Account
  domains: Domain[]
  isExpanded: boolean
  selectedDomainId: string | null
  onToggle: () => void
  onSelectDomain: (id: string) => void
  onDelete: () => void
  isLoading?: boolean
  isLoadingMore?: boolean
  hasMore?: boolean
  onLoadMore?: () => void
}

export function AccountTreeItem({
  account,
  domains,
  isExpanded,
  selectedDomainId,
  onToggle,
  onSelectDomain,
  onDelete,
  isLoading = false,
  isLoadingMore = false,
  hasMore = false,
  onLoadMore,
}: AccountTreeItemProps) {
  const { t } = useTranslation()
  const hasError = account.status === "error"

  return (
    <Collapsible open={isExpanded} onOpenChange={onToggle}>
      <div className="rounded-lg border bg-card">
        {/* 账户行 */}
        <div className={cn("group flex items-center gap-1 p-1", hasError && "opacity-60")}>
          <CollapsibleTrigger asChild disabled={hasError}>
            <button
              type="button"
              className={cn(
                "flex flex-1 items-center gap-2 rounded-md px-2 py-2 text-left transition-colors",
                "hover:bg-sidebar-accent",
                hasError && "cursor-not-allowed"
              )}
            >
              <ChevronRight
                className={cn(
                  "h-4 w-4 shrink-0 text-muted-foreground transition-transform",
                  isExpanded && "rotate-90"
                )}
              />
              <ProviderIcon provider={account.provider} className="h-4 w-4 shrink-0" />
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-sm">{account.name}</div>
                <div className="text-muted-foreground text-xs">
                  {getProviderName(account.provider)}
                </div>
              </div>
              {hasError && (
                <span title={account.error || t("account.loadFailed")}>
                  <TriangleAlert className="h-4 w-4 shrink-0 text-destructive" />
                </span>
              )}
            </button>
          </CollapsibleTrigger>

          {/* 更多操作 */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 shrink-0 opacity-0 transition-opacity group-hover:opacity-100"
                onClick={(e) => e.stopPropagation()}
              >
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem
                onClick={onDelete}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="mr-2 h-4 w-4" />
                {t("account.deleteAccount")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>

        {/* 域名列表 */}
        <CollapsibleContent>
          <div className="border-t px-2 py-2">
            <div className="ml-4 space-y-0.5 border-l pl-2">
              {isLoading ? (
                <div className="space-y-1 py-1">
                  <Skeleton className="h-7 w-full" />
                  <Skeleton className="h-7 w-full" />
                  <Skeleton className="h-7 w-3/4" />
                </div>
              ) : domains.length === 0 ? (
                <div className="py-2 text-muted-foreground text-xs">{t("domain.noDomains")}</div>
              ) : (
                <>
                  {domains.map((domain) => (
                    <DomainTreeItem
                      key={domain.id}
                      domain={domain}
                      isSelected={selectedDomainId === domain.id}
                      onSelect={() => onSelectDomain(domain.id)}
                    />
                  ))}
                  {/* 加载更多 */}
                  {hasMore && (
                    <button
                      type="button"
                      onClick={onLoadMore}
                      disabled={isLoadingMore}
                      className="flex w-full items-center justify-center gap-1 py-1.5 text-muted-foreground text-xs transition-colors hover:text-foreground"
                    >
                      {isLoadingMore ? (
                        <Loader2 className="h-3 w-3 animate-spin" />
                      ) : (
                        t("common.loadMore")
                      )}
                    </button>
                  )}
                </>
              )}
            </div>
          </div>
        </CollapsibleContent>
      </div>
    </Collapsible>
  )
}
