/**
 * 导航相关类型定义
 */

/** 导航项标识 */
export type NavItem = "main" | "domains" | "favorites" | "toolbox" | "settings" | "accounts"

/** 导航路径映射 */
export const NAV_PATHS: Record<NavItem, string> = {
  main: "/",
  domains: "/domains",
  favorites: "/favorites",
  toolbox: "/toolbox",
  settings: "/settings",
  accounts: "/accounts",
} as const

/**
 * 从路由路径推断导航项
 * @param pathname - 路由路径
 * @returns 对应的导航项
 */
export function getNavItemFromPath(pathname: string): NavItem {
  if (pathname === "/") return "main"
  if (pathname.startsWith("/domains")) return "domains"
  if (pathname.startsWith("/favorites")) return "favorites"
  if (pathname.startsWith("/toolbox")) return "toolbox"
  if (pathname.startsWith("/settings")) return "settings"
  if (pathname.startsWith("/accounts")) return "accounts"
  return "main"
}
