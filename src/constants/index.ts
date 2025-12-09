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
} as const

/** DNS 配置 */
export const DNS = {
  /** 默认 TTL（秒） */
  DEFAULT_TTL: 300,
} as const

/** UI 配置 */
export const UI = {
  /** 错误消息最大长度 */
  MAX_ERROR_MESSAGE_LENGTH: 300,
} as const
