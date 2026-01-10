/**
 * Storage Key 常量和默认值
 */

import type { StorageKey, StorageMap } from "./types"

/** 应用存储前缀 */
export const STORAGE_PREFIX = "dns-orchestrator:"

/**
 * Storage Key 到 localStorage key 的映射
 * 统一添加前缀，避免与其他应用冲突
 */
export const STORAGE_KEYS: Record<StorageKey, string> = {
  theme: `${STORAGE_PREFIX}theme`,
  language: `${STORAGE_PREFIX}language`,
  debugMode: `${STORAGE_PREFIX}debugMode`,
  sidebarCollapsed: `${STORAGE_PREFIX}sidebarCollapsed`,
  paginationMode: `${STORAGE_PREFIX}paginationMode`,
  showRecordHints: `${STORAGE_PREFIX}showRecordHints`,
  operationNotifications: `${STORAGE_PREFIX}operationNotifications`,
  recentDomains: `${STORAGE_PREFIX}recentDomains`,
  domainsCache: `${STORAGE_PREFIX}domainsCache`,
  skippedVersion: `${STORAGE_PREFIX}skippedVersion`,
} as const

/**
 * 默认值
 */
export const STORAGE_DEFAULTS: StorageMap = {
  theme: "system",
  language: "zh-CN",
  debugMode: false,
  sidebarCollapsed: false,
  paginationMode: "infinite",
  showRecordHints: false,
  operationNotifications: true,
  recentDomains: [],
  domainsCache: { domainsByAccount: {}, scrollPosition: 0 },
  skippedVersion: null,
}

/**
 * 旧 key 到新 key 的映射（用于数据迁移）
 */
export const LEGACY_KEYS: Record<string, StorageKey> = {
  theme: "theme",
  language: "language",
  debugMode: "debugMode",
  sidebarCollapsed: "sidebarCollapsed",
  paginationMode: "paginationMode",
  showRecordHints: "showRecordHints",
  operationNotifications: "operationNotifications",
  recent_domains: "recentDomains",
  "dns-orchestrator-domains-cache": "domainsCache",
  "dns-orchestrator-skipped-version": "skippedVersion",
}
