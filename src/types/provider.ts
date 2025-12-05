/** 凭证字段定义 */
export interface ProviderCredentialField {
  key: string
  label: string
  type: "text" | "password"
  placeholder?: string
  helpText?: string
}

/** 提供商支持的功能 */
export interface ProviderFeatures {
  /** 是否支持代理功能 (如 Cloudflare 的 CDN 代理) */
  proxy: boolean
}

/** 提供商信息 (从后端获取) */
export interface ProviderInfo {
  id: string
  name: string
  description: string
  requiredFields: ProviderCredentialField[]
  features: ProviderFeatures
}
