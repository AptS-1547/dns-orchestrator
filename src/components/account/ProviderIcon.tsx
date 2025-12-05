import { Cloud, Globe, Server } from "lucide-react"

interface ProviderIconProps {
  provider: string
  className?: string
}

// 提供商图标映射
const PROVIDER_ICONS: Record<string, React.ComponentType<{ className?: string }>> = {
  cloudflare: Cloud,
  aliyun: Server,
  dnspod: Server,
  huaweicloud: Server,
}

export function ProviderIcon({ provider, className }: ProviderIconProps) {
  const Icon = PROVIDER_ICONS[provider] || Globe
  return <Icon className={className} />
}

// 备用函数：当无法从 store 获取名称时使用
export function getProviderName(provider: string): string {
  const PROVIDER_NAMES: Record<string, string> = {
    cloudflare: "Cloudflare",
    aliyun: "阿里云",
    dnspod: "DNSPod",
    huaweicloud: "华为云",
  }
  return PROVIDER_NAMES[provider] || provider
}
