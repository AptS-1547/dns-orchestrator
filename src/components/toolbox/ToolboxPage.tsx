import { ArrowLeft, FileText, Globe, Lock, MapPin, Wrench } from "lucide-react"
import { useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router-dom"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { cn } from "@/lib/utils"
import { DnsLookup } from "./DnsLookup"
import { IpLookup } from "./IpLookup"
import { SslCheck } from "./SslCheck"
import { WhoisLookup } from "./WhoisLookup"

const TABS = [
  { id: "dns", icon: Globe, label: "DNS" },
  { id: "whois", icon: FileText, label: "WHOIS" },
  { id: "ssl", icon: Lock, label: "SSL" },
  { id: "ip", icon: MapPin, label: "IP" },
] as const

export function ToolboxPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
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
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-3 border-b bg-background px-4 py-3 sm:gap-4 sm:px-6 sm:py-4">
        <Button variant="ghost" size="icon" onClick={() => navigate("/")}>
          <ArrowLeft className="h-5 w-5" />
        </Button>
        <div className="flex items-center gap-2">
          <Wrench className="h-5 w-5 text-primary" />
          <h2 className="font-semibold text-xl">{t("toolbox.title")}</h2>
        </div>
      </div>

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
          <div className="mx-auto max-w-4xl p-4 sm:p-6">
            <TabsContent value="dns" className="fade-in-0 mt-0 animate-in duration-200">
              <DnsLookup />
            </TabsContent>
            <TabsContent value="whois" className="fade-in-0 mt-0 animate-in duration-200">
              <WhoisLookup />
            </TabsContent>
            <TabsContent value="ssl" className="fade-in-0 mt-0 animate-in duration-200">
              <SslCheck />
            </TabsContent>
            <TabsContent value="ip" className="fade-in-0 mt-0 animate-in duration-200">
              <IpLookup />
            </TabsContent>
          </div>
        </ScrollArea>
      </Tabs>
    </div>
  )
}
