/**
 * Storage 层类型定义
 * 抽象 localStorage 的统一接口，参考 Transport 层设计
 */

import type { LanguageCode } from "@/i18n"

// ============ 数据类型定义 ============

/** 主题类型 */
export type Theme = "light" | "dark" | "system"

/** 分页模式 */
export type PaginationMode = "infinite" | "paginated"

/** 最近访问的域名 */
export interface RecentDomain {
  accountId: string
  domainId: string
  domainName: string
  accountName: string
  provider: string
  timestamp: number
}

/** 域名缓存数据 */
export interface DomainsCacheData {
  domainsByAccount: Record<string, AccountDomainCache>
  scrollPosition: number
}

/** 单个账户的域名缓存 */
interface AccountDomainCache {
  domains: unknown[] // 避免循环引用，实际类型为 Domain[]
  lastUpdated: number
  page: number
  hasMore: boolean
}

// ============ Storage Key 映射 ============

/**
 * Storage Key 到类型的映射
 * 类似 Transport 的 CommandMap
 */
export interface StorageMap {
  /** 主题设置 */
  theme: Theme
  /** 语言设置 */
  language: LanguageCode
  /** 调试模式 */
  debugMode: boolean
  /** 侧边栏折叠状态 */
  sidebarCollapsed: boolean
  /** 分页模式 */
  paginationMode: PaginationMode
  /** 显示记录提示 */
  showRecordHints: boolean
  /** 操作通知 */
  operationNotifications: boolean
  /** 最近访问域名 */
  recentDomains: RecentDomain[]
  /** 域名缓存 */
  domainsCache: DomainsCacheData
  /** 跳过的更新版本 */
  skippedVersion: string | null
}

// ============ 类型工具 ============

/** 提取 key 的值类型 */
export type StorageValue<K extends keyof StorageMap> = StorageMap[K]

/** 所有 Storage Key */
export type StorageKey = keyof StorageMap

// ============ Storage 接口 ============

/**
 * Storage 抽象接口
 * 类似 ITransport 的设计
 */
export interface IStorage {
  /**
   * 获取存储值
   * @param key - 存储键
   * @returns 存储值，不存在时返回 null
   */
  get<K extends StorageKey>(key: K): StorageValue<K> | null

  /**
   * 获取存储值（带默认值）
   * @param key - 存储键
   * @param defaultValue - 默认值
   * @returns 存储值或默认值
   */
  getWithDefault<K extends StorageKey>(key: K, defaultValue: StorageValue<K>): StorageValue<K>

  /**
   * 设置存储值
   * @param key - 存储键
   * @param value - 要存储的值
   */
  set<K extends StorageKey>(key: K, value: StorageValue<K>): void

  /**
   * 删除存储值
   * @param key - 存储键
   */
  remove<K extends StorageKey>(key: K): void

  /**
   * 检查 key 是否存在
   * @param key - 存储键
   */
  has<K extends StorageKey>(key: K): boolean

  /**
   * 清空所有应用存储
   */
  clear(): void
}
