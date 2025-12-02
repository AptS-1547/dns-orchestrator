import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AppLayout } from "@/components/layout/AppLayout";
import { SettingsPage } from "@/components/settings/SettingsPage";
import { ToolboxPage } from "@/components/toolbox/ToolboxPage";
import { StatusBar } from "@/components/ui/status-bar";
import { Toaster } from "@/components/ui/sonner";
import { initTheme, useDomainStore } from "@/stores";
import { useUpdaterStore } from "@/stores/updaterStore";

type View = "main" | "settings" | "toolbox";

function App() {
  const { t } = useTranslation();
  const { checkForUpdates } = useUpdaterStore();
  const { selectDomain } = useDomainStore();
  const [currentView, setCurrentView] = useState<View>("main");

  useEffect(() => {
    initTheme();

    // 启动后延迟 3 秒检查更新（静默检查，状态栏会自动显示）
    const timer = setTimeout(async () => {
      try {
        await checkForUpdates();
      } catch (error) {
        // 启动时的错误检查不打扰用户，仅记录日志
        console.error("Update check failed:", error);
      }
    }, 3000);

    return () => clearTimeout(timer);
  }, [checkForUpdates, t]);

  return (
    <>
      <AppLayout
        onOpenToolbox={() => {
          selectDomain(null);
          setCurrentView("toolbox");
        }}
        onNavigateToMain={() => setCurrentView("main")}
      >
        {currentView === "settings" ? (
          <SettingsPage onBack={() => setCurrentView("main")} />
        ) : currentView === "toolbox" ? (
          <ToolboxPage onBack={() => setCurrentView("main")} />
        ) : null}
      </AppLayout>
      <StatusBar
        onOpenSettings={() => {
          selectDomain(null);
          setCurrentView("settings");
        }}
      />
      <Toaster richColors position="top-right" />
    </>
  );
}

export default App;
