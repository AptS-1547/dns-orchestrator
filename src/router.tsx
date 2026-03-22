/**
 * 路由配置
 * 所有页面组件使用 lazy 懒加载，按路由拆分 chunk
 */

import { createBrowserRouter, Navigate } from "react-router-dom"
import { RootLayout } from "@/components/layout/RootLayout"

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      {
        index: true,
        lazy: () => import("@/components/home/HomePage").then((m) => ({ Component: m.HomePage })),
      },
      {
        path: "domains",
        lazy: () =>
          import("@/components/domains/DomainSelectorPage").then((m) => ({
            Component: m.DomainSelectorPage,
          })),
      },
      {
        path: "domains/:accountId/:domainId",
        lazy: () =>
          import("@/components/domains/DnsRecordPage").then((m) => ({
            Component: m.DnsRecordPage,
          })),
      },
      {
        path: "favorites",
        lazy: () =>
          import("@/components/domains/FavoriteDomainsPage").then((m) => ({
            Component: m.FavoriteDomainsPage,
          })),
      },
      {
        path: "accounts",
        lazy: () =>
          import("@/components/accounts/AccountsPage").then((m) => ({
            Component: m.AccountsPage,
          })),
      },
      {
        path: "settings",
        lazy: () =>
          import("@/components/settings/SettingsPage").then((m) => ({
            Component: m.SettingsPage,
          })),
      },
      {
        path: "toolbox",
        lazy: () =>
          import("@/components/toolbox/ToolboxPage").then((m) => ({
            Component: m.ToolboxPage,
          })),
      },
      {
        // 404 重定向到首页
        path: "*",
        element: <Navigate to="/" replace />,
      },
    ],
  },
])
