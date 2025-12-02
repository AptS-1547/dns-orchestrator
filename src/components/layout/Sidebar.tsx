import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAccountStore, useDomainStore } from "@/stores";
import { AccountList } from "@/components/account/AccountList";
import { AccountForm } from "@/components/account/AccountForm";
import { DomainList } from "@/components/domain/DomainList";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Skeleton } from "@/components/ui/skeleton";
import { Globe, Plus, Wrench } from "lucide-react";

interface SidebarProps {
  onOpenToolbox?: () => void;
  onNavigateToMain?: () => void;
}

export function Sidebar({ onOpenToolbox, onNavigateToMain }: SidebarProps) {
  const { t } = useTranslation();
  const {
    accounts,
    selectedAccountId,
    isLoading: isAccountLoading,
    isDeleting: isAccountDeleting,
    fetchAccounts,
    selectAccount,
    deleteAccount,
  } = useAccountStore();
  const {
    domains,
    selectedDomainId,
    isLoading: isDomainLoading,
    fetchDomains,
    selectDomain,
    clearDomains,
  } = useDomainStore();

  const [showAccountForm, setShowAccountForm] = useState(false);

  // 切换账户时先清除域名选择，避免用旧的 domainId 请求新账户
  const handleSelectAccount = useCallback(
    (id: string | null) => {
      if (id !== selectedAccountId) {
        selectDomain(null);
      }
      selectAccount(id);
    },
    [selectedAccountId, selectAccount, selectDomain]
  );

  useEffect(() => {
    fetchAccounts();
  }, [fetchAccounts]);

  useEffect(() => {
    if (selectedAccountId) {
      fetchDomains(selectedAccountId);
    } else {
      clearDomains();
    }
  }, [selectedAccountId, fetchDomains, clearDomains]);

  return (
    <aside className="w-64 border-r bg-sidebar flex flex-col">
      {/* Header */}
      <div className="p-4 border-b">
        <div className="flex items-center gap-2">
          <Globe className="h-6 w-6 text-primary" />
          <h1 className="text-lg font-semibold">{t("common.appName")}</h1>
        </div>
      </div>

      <ScrollArea className="flex-1">
        {/* 账号列表 */}
        <div className="p-4">
          <div className="flex items-center justify-between mb-3">
            <h2 className="text-sm font-medium text-muted-foreground">{t("account.title")}</h2>
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6"
              onClick={() => setShowAccountForm(true)}
            >
              <Plus className="h-4 w-4" />
            </Button>
          </div>
          {isAccountLoading ? (
            <div className="space-y-2">
              <Skeleton className="h-10 w-full" />
              <Skeleton className="h-10 w-full" />
            </div>
          ) : (
            <AccountList
              accounts={accounts}
              selectedId={selectedAccountId}
              onSelect={handleSelectAccount}
              onDelete={deleteAccount}
              isDeleting={isAccountDeleting}
            />
          )}
        </div>

        {/* 域名列表 */}
        {selectedAccountId && (
          <>
            <Separator />
            <div className="p-4">
              <h2 className="text-sm font-medium text-muted-foreground mb-3">
                {t("domain.title")}
              </h2>
              {isDomainLoading ? (
                <div className="space-y-2">
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                  <Skeleton className="h-8 w-full" />
                </div>
              ) : (
                <DomainList
                  domains={domains}
                  selectedId={selectedDomainId}
                  onSelect={(id) => {
                    selectDomain(id);
                    onNavigateToMain?.();
                  }}
                />
              )}
            </div>
          </>
        )}
      </ScrollArea>

      {/* 底部工具箱按钮 */}
      <Separator />
      <div className="p-3">
        <Button
          variant="ghost"
          className="w-full justify-start gap-2"
          onClick={onOpenToolbox}
        >
          <Wrench className="h-4 w-4" />
          {t("toolbox.title")}
        </Button>
      </div>

      {/* Dialogs */}
      <AccountForm open={showAccountForm} onOpenChange={setShowAccountForm} />
    </aside>
  );
}
