/**
 * localStorage 实现
 */

import { logger } from "@/lib/logger"
import { LEGACY_KEYS, STORAGE_KEYS, STORAGE_PREFIX } from "./keys"
import type { IStorage, StorageKey, StorageValue } from "./types"

/**
 * 判断值是否需要 JSON 序列化
 */
function needsSerialization(value: unknown): boolean {
  return typeof value === "object" && value !== null
}

/**
 * 迁移旧数据到新格式
 * 在应用启动时调用一次
 */
function migrateStorage(): void {
  const migratedKey = `${STORAGE_PREFIX}migrated`
  const migrated = localStorage.getItem(migratedKey)
  if (migrated === "v1") return

  // 迁移每个旧 key
  for (const [oldKey, newKey] of Object.entries(LEGACY_KEYS)) {
    const oldValue = localStorage.getItem(oldKey)
    if (oldValue !== null) {
      const newStorageKey = STORAGE_KEYS[newKey]
      // 只在新 key 不存在时迁移
      if (localStorage.getItem(newStorageKey) === null) {
        localStorage.setItem(newStorageKey, oldValue)
        logger.debug(`[Storage] Migrated "${oldKey}" to "${newStorageKey}"`)
      }
      // 迁移完成后移除旧 key，避免重复数据
      localStorage.removeItem(oldKey)
      logger.debug(`[Storage] Removed legacy key "${oldKey}"`)
    }
  }

  localStorage.setItem(migratedKey, "v1")
  logger.debug("[Storage] Migration completed")
}

/**
 * localStorage 适配器
 */
class LocalStorageAdapter implements IStorage {
  constructor() {
    // 初始化时执行迁移
    migrateStorage()
  }

  get<K extends StorageKey>(key: K): StorageValue<K> | null {
    try {
      const raw = localStorage.getItem(STORAGE_KEYS[key])
      if (raw === null) return null

      // 尝试 JSON 解析
      try {
        return JSON.parse(raw) as StorageValue<K>
      } catch {
        // 非 JSON，返回原始字符串
        return raw as StorageValue<K>
      }
    } catch (err) {
      logger.warn(`[Storage] Failed to get "${key}":`, err)
      return null
    }
  }

  getWithDefault<K extends StorageKey>(key: K, defaultValue: StorageValue<K>): StorageValue<K> {
    const value = this.get(key)
    return value !== null ? value : defaultValue
  }

  set<K extends StorageKey>(key: K, value: StorageValue<K>): void {
    try {
      const serialized = needsSerialization(value) ? JSON.stringify(value) : String(value)
      localStorage.setItem(STORAGE_KEYS[key], serialized)
    } catch (err) {
      logger.error(`[Storage] Failed to set "${key}":`, err)
    }
  }

  remove<K extends StorageKey>(key: K): void {
    try {
      localStorage.removeItem(STORAGE_KEYS[key])
    } catch (err) {
      logger.warn(`[Storage] Failed to remove "${key}":`, err)
    }
  }

  has<K extends StorageKey>(key: K): boolean {
    return localStorage.getItem(STORAGE_KEYS[key]) !== null
  }

  clear(): void {
    // 只清理 dns-orchestrator 前缀的 key
    const keysToRemove: string[] = []

    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i)
      if (key?.startsWith(STORAGE_PREFIX)) {
        keysToRemove.push(key)
      }
    }

    keysToRemove.forEach((key) => localStorage.removeItem(key))
    logger.debug(`[Storage] Cleared ${keysToRemove.length} keys`)
  }
}

/** Storage 单例 */
export const storage: IStorage = new LocalStorageAdapter()
