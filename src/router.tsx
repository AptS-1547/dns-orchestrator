/**
 * 路由配置
 */

import { createBrowserRouter, Navigate } from "react-router-dom"
import { AccountsPage } from "@/components/accounts/AccountsPage"
import { DnsRecordPage } from "@/components/domains/DnsRecordPage"
import { DomainSelectorPage } from "@/components/domains/DomainSelectorPage"
import { FavoriteDomainsPage } from "@/components/domains/FavoriteDomainsPage"
import { HomePage } from "@/components/home/HomePage"
import { RootLayout } from "@/components/layout/RootLayout"
import { SettingsPage } from "@/components/settings/SettingsPage"
import { ToolboxPage } from "@/components/toolbox/ToolboxPage"

export const router = createBrowserRouter([
  {
    path: "/",
    element: <RootLayout />,
    children: [
      {
        index: true,
        element: <HomePage />,
      },
      {
        path: "domains",
        element: <DomainSelectorPage />,
      },
      {
        path: "domains/:accountId/:domainId",
        element: <DnsRecordPage />,
      },
      {
        path: "favorites",
        element: <FavoriteDomainsPage />,
      },
      {
        path: "accounts",
        element: <AccountsPage />,
      },
      {
        path: "settings",
        element: <SettingsPage />,
      },
      {
        path: "toolbox",
        element: <ToolboxPage />,
      },
      {
        // 404 重定向到首页
        path: "*",
        element: <Navigate to="/" replace />,
      },
    ],
  },
])
