//! 域名颜色标记预设配置

/**
 * 域名颜色定义
 * - hex: 浅色模式颜色
 * - darkHex: 暗色模式颜色
 * - name: 颜色名称（用于 UI 显示）
 */
export const DOMAIN_COLORS = {
  red: { hex: "#ef4444", darkHex: "#f87171", name: "红色" },
  orange: { hex: "#f97316", darkHex: "#fb923c", name: "橙色" },
  yellow: { hex: "#eab308", darkHex: "#fbbf24", name: "黄色" },
  green: { hex: "#22c55e", darkHex: "#4ade80", name: "绿色" },
  teal: { hex: "#14b8a6", darkHex: "#2dd4bf", name: "青色" },
  blue: { hex: "#3b82f6", darkHex: "#60a5fa", name: "蓝色" },
  purple: { hex: "#a855f7", darkHex: "#c084fc", name: "紫色" },
  pink: { hex: "#ec4899", darkHex: "#f472b6", name: "粉色" },
  brown: { hex: "#a16207", darkHex: "#ca8a04", name: "棕色" },
  gray: { hex: "#6b7280", darkHex: "#9ca3af", name: "灰色" },
} as const

/**
 * 域名颜色键类型
 */
export type DomainColorKey = keyof typeof DOMAIN_COLORS

/**
 * 无颜色常量
 */
export const DOMAIN_COLOR_NONE = "none"

/**
 * 域名颜色值类型（包含 "none" 表示无颜色）
 */
export type DomainColorValue = DomainColorKey | "none"

/**
 * 颜色排序（用于 UI 显示顺序）
 */
export const DOMAIN_COLOR_ORDER: DomainColorKey[] = [
  "red",
  "orange",
  "yellow",
  "green",
  "teal",
  "blue",
  "purple",
  "pink",
  "brown",
  "gray",
]

/**
 * 获取当前主题下的颜色值
 * @param colorKey - 颜色键（"none" 表示无颜色）
 * @param isDark - 是否为暗色模式
 * @returns HEX 颜色值，如果 colorKey 为 "none" 则返回 undefined
 */
export function getDomainColor(
  colorKey: DomainColorValue | undefined | null,
  isDark: boolean
): string | undefined {
  if (!colorKey || colorKey === DOMAIN_COLOR_NONE) return undefined
  const color = DOMAIN_COLORS[colorKey as DomainColorKey]
  return isDark ? color.darkHex : color.hex
}
