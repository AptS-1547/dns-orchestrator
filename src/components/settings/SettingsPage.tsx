import { Settings } from "lucide-react"
import { useTranslation } from "react-i18next"
import { PageContainer } from "@/components/ui/page-container"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { AboutTab } from "./tabs/AboutTab"
import { AppearanceTab } from "./tabs/AppearanceTab"
import { FeaturesTab } from "./tabs/FeaturesTab"

/**
 * 设置页面
 * 使用 Tabs 布局将设置分为：外观、功能、关于三个标签页
 */
export function SettingsPage() {
  const { t } = useTranslation()

  return (
    <PageLayout>
      <PageHeader title={t("settings.title")} icon={<Settings className="h-5 w-5" />} />

      <PageContainer maxWidth="max-w-3xl">
        <Tabs defaultValue="appearance" className="space-y-6">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="appearance">{t("settings.tabs.appearance")}</TabsTrigger>
            <TabsTrigger value="features">{t("settings.tabs.features")}</TabsTrigger>
            <TabsTrigger value="about">{t("settings.tabs.about")}</TabsTrigger>
          </TabsList>

          <TabsContent value="appearance">
            <AppearanceTab />
          </TabsContent>

          <TabsContent value="features">
            <FeaturesTab />
          </TabsContent>

          <TabsContent value="about">
            <AboutTab />
          </TabsContent>
        </Tabs>
      </PageContainer>
    </PageLayout>
  )
}
