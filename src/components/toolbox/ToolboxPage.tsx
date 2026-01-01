import { FileText, Globe, Lock, MapPin, Network, Wrench } from "lucide-react"
import { useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { PageHeader } from "@/components/ui/page-header"
import { PageLayout } from "@/components/ui/page-layout"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { cn } from "@/lib/utils"
import { DnsLookup } from "./DnsLookup"
import { HttpHeaderCheck } from "./HttpHeaderCheck"
import { IpLookup } from "./IpLookup"
import { SslCheck } from "./SslCheck"
import { WhoisLookup } from "./WhoisLookup"

const TABS = [
  { id: "dns", icon: Globe, label: "DNS" },
  { id: "whois", icon: FileText, label: "WHOIS" },
  { id: "ssl", icon: Lock, label: "SSL" },
  { id: "http", icon: Network, label: "HTTP" },
  { id: "ip", icon: MapPin, label: "IP" },
] as const

export function ToolboxPage() {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState("dns")
  const [indicatorStyle, setIndicatorStyle] = useState({ left: 0, width: 0 })
  const tabsRef = useRef<Map<string, HTMLButtonElement>>(new Map())

  // 更新指示器位置
  useEffect(() => {
    const activeElement = tabsRef.current.get(activeTab)
    if (activeElement) {
      const parent = activeElement.parentElement
      if (parent) {
        const parentRect = parent.getBoundingClientRect()
        const activeRect = activeElement.getBoundingClientRect()
        setIndicatorStyle({
          left: activeRect.left - parentRect.left,
          width: activeRect.width,
        })
      }
    }
  }, [activeTab])

  return (
    <PageLayout>
      <PageHeader title={t("toolbox.title")} icon={<Wrench className="h-5 w-5" />} />

      {/* Tabs */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="flex min-h-0 flex-1 flex-col">
        <div className="overflow-x-auto border-b px-4 sm:px-6">
          <TabsList className="relative h-auto flex-nowrap gap-1 bg-transparent p-0">
            {/* 滑动指示器 */}
            <div
              className="absolute bottom-0 h-0.5 bg-primary transition-all duration-300 ease-out"
              style={{
                left: indicatorStyle.left,
                width: indicatorStyle.width,
              }}
            />
            {TABS.map(({ id, icon: Icon, label }) => (
              <TabsTrigger
                key={id}
                ref={(el) => {
                  if (el) tabsRef.current.set(id, el)
                }}
                value={id}
                className={cn(
                  "gap-1.5 rounded-none border-transparent border-b-2 px-3 py-2.5",
                  "transition-colors duration-200",
                  "data-[state=active]:bg-transparent data-[state=active]:shadow-none"
                )}
              >
                <Icon className="h-4 w-4" />
                {label}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>

        <ScrollArea className="min-h-0 flex-1">
          <div className="scroll-pb-safe mx-auto max-w-4xl p-4 sm:p-6">
            <TabsContent value="dns" className="fade-in-0 mt-0 animate-in duration-200">
              <DnsLookup />
            </TabsContent>
            <TabsContent value="whois" className="fade-in-0 mt-0 animate-in duration-200">
              <WhoisLookup />
            </TabsContent>
            <TabsContent value="ssl" className="fade-in-0 mt-0 animate-in duration-200">
              <SslCheck />
            </TabsContent>
            <TabsContent value="http" className="fade-in-0 mt-0 animate-in duration-200">
              <HttpHeaderCheck />
            </TabsContent>
            <TabsContent value="ip" className="fade-in-0 mt-0 animate-in duration-200">
              <IpLookup />
            </TabsContent>
          </div>
        </ScrollArea>
      </Tabs>
    </PageLayout>
  )
}
