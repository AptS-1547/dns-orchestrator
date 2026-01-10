/** 分页配置 */
export const PAGINATION = {
  PAGE_SIZE: 20,
} as const

/** 时间配置（毫秒） */
export const TIMING = {
  /** 搜索输入防抖延迟 */
  DEBOUNCE_DELAY: 300,
  /** Tooltip 显示延迟 */
  TOOLTIP_DELAY: 300,
  /** Toast 持续时间 */
  TOAST_DURATION: 5000,
  /** 更新检查延迟 */
  UPDATE_CHECK_DELAY: 3000,
  /** 滚动位置保存防抖延迟 */
  SCROLL_SAVE_DEBOUNCE: 300,
  /** 更新重试延迟 */
  UPDATE_RETRY_DELAY: 2000,
} as const

/** DNS 配置 */
export const DNS = {
  /** 默认 TTL（秒） */
  DEFAULT_TTL: 300,
  /** 默认 MX 优先级 */
  DEFAULT_MX_PRIORITY: 10,
} as const

/** UI 配置 */
export const UI = {
  /** 错误消息最大长度 */
  MAX_ERROR_MESSAGE_LENGTH: 300,
} as const

/** 业务限制 */
export const LIMITS = {
  MAX_RECENT_DOMAINS: 6,
} as const

/** 网络配置 */
export const NETWORK = {
  MIN_PORT: 1,
  MAX_PORT: 65535,
  DEFAULT_HTTPS_PORT: 443,
} as const

/** 外部链接 */
export const EXTERNAL_LINKS = {
  GITHUB_REPO: "https://github.com/AptS-1547/dns-orchestrator",
  GITHUB_RELEASES: "https://github.com/AptS-1547/dns-orchestrator/releases/latest",
} as const

/** DNS 服务器列表 */
export const DNS_SERVERS = [
  { label: "toolbox.systemDefault", value: "system", isRaw: false },
  { label: "Google (8.8.8.8)", value: "8.8.8.8", isRaw: true },
  { label: "Cloudflare (1.1.1.1)", value: "1.1.1.1", isRaw: true },
  { label: "AliDNS (223.5.5.5)", value: "223.5.5.5", isRaw: true },
  { label: "Tencent (119.29.29.29)", value: "119.29.29.29", isRaw: true },
  { label: "toolbox.custom", value: "custom", isRaw: false },
] as const
