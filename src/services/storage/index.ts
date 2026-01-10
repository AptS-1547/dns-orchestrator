/**
 * Storage 层统一导出
 */

export { storage } from "./localStorage.storage"
export { STORAGE_DEFAULTS, STORAGE_KEYS, STORAGE_PREFIX } from "./keys"
export type {
  DomainsCacheData,
  IStorage,
  PaginationMode,
  RecentDomain,
  StorageKey,
  StorageMap,
  StorageValue,
  Theme,
} from "./types"
