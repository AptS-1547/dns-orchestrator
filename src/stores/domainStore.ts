import { create } from "zustand"
import { PAGINATION } from "@/constants"
import { extractErrorMessage, getErrorMessage, isCredentialError } from "@/lib/error"
import { logger } from "@/lib/logger"
import { domainService } from "@/services"
import { type DomainsCacheData, storage } from "@/services/storage"
import { useAccountStore } from "./accountStore"
import type { AccountDomainCache, DomainState } from "./domainStore.types"
import { createDomainBatchSlice } from "./slices/domainBatchSlice"
import { createDomainFilterSlice } from "./slices/domainFilterSlice"
import { createDomainMetadataSlice } from "./slices/domainMetadataSlice"
import { createDomainUiSlice } from "./slices/domainUiSlice"

// Re-export types for consumers
export type { FavoriteDomain } from "./domainStore.types"

// 从 storage 读取初始缓存数据
function getInitialCache(): {
  domainsByAccount: Record<string, AccountDomainCache>
  scrollPosition: number
} {
  try {
    const cached = storage.get("domainsCache")
    if (cached) {
      if ("domainsByAccount" in cached) {
        return {
          domainsByAccount: (cached as DomainsCacheData).domainsByAccount as Record<
            string,
            AccountDomainCache
          >,
          scrollPosition: (cached as DomainsCacheData).scrollPosition ?? 0,
        }
      }
      return {
        domainsByAccount: cached as unknown as Record<string, AccountDomainCache>,
        scrollPosition: 0,
      }
    }
  } catch (err) {
    logger.warn("Failed to load domain cache, clearing:", err)
    storage.remove("domainsCache")
  }
  return { domainsByAccount: {}, scrollPosition: 0 }
}

const initialCache = getInitialCache()

const getDomainPageSize = (accountId: string): number => {
  const { accounts, providers } = useAccountStore.getState()
  const account = accounts.find((a) => a.id === accountId)
  if (!account) return PAGINATION.PAGE_SIZE

  const provider = providers.find((p) => p.id === account.provider)
  const maxPageSize = provider?.limits.maxPageSizeDomains ?? 100

  return Math.min(PAGINATION.PAGE_SIZE, maxPageSize)
}

export const useDomainStore = create<DomainState>((set, get) => ({
  // ===== 初始状态 =====
  domainsByAccount: initialCache.domainsByAccount,
  selectedAccountId: null,
  selectedDomainId: null,
  loadingAccounts: new Set(),
  loadingMoreAccounts: new Set(),
  isBackgroundRefreshing: false,
  expandedAccounts: new Set(),
  scrollPosition: initialCache.scrollPosition,
  selectedTags: new Set(),
  selectedDomainKeys: new Set(),
  isBatchMode: false,
  isBatchOperating: false,

  // ===== 核心方法 =====

  loadFromStorage: () => {
    try {
      const cached = storage.get("domainsCache")
      if (cached) {
        if ("domainsByAccount" in cached) {
          set({
            domainsByAccount: (cached as DomainsCacheData).domainsByAccount as Record<
              string,
              AccountDomainCache
            >,
            scrollPosition: (cached as DomainsCacheData).scrollPosition ?? 0,
          })
        } else {
          set({
            domainsByAccount: cached as unknown as Record<string, AccountDomainCache>,
            scrollPosition: 0,
          })
        }
      }
    } catch (err) {
      logger.error("Failed to load domain cache from storage:", err)
    }
  },

  saveToStorage: () => {
    try {
      const { domainsByAccount, scrollPosition } = get()
      storage.set("domainsCache", { domainsByAccount, scrollPosition } as DomainsCacheData)
    } catch (err) {
      logger.error("Failed to save domain cache to storage:", err)
    }
  },

  refreshAllAccounts: async (accounts) => {
    const { isBackgroundRefreshing } = get()
    if (isBackgroundRefreshing) return

    set({ isBackgroundRefreshing: true })

    try {
      await Promise.allSettled(
        accounts.map(async (account) => {
          try {
            const pageSize = getDomainPageSize(account.id)
            const response = await domainService.listDomains(account.id, 1, pageSize)
            if (response.success && response.data) {
              set((state) => ({
                domainsByAccount: {
                  ...state.domainsByAccount,
                  [account.id]: {
                    domains: response.data?.items ?? [],
                    lastUpdated: Date.now(),
                    page: response.data?.page ?? 1,
                    hasMore: response.data?.hasMore ?? false,
                  },
                },
              }))
            }
          } catch {
            // 静默失败
          }
        })
      )
      get().saveToStorage()
    } finally {
      set({ isBackgroundRefreshing: false })
    }
  },

  refreshAccount: async (accountId) => {
    const { loadingAccounts } = get()
    if (loadingAccounts.has(accountId)) return

    set((state) => ({
      loadingAccounts: new Set(state.loadingAccounts).add(accountId),
    }))

    try {
      const pageSize = getDomainPageSize(accountId)
      const response = await domainService.listDomains(accountId, 1, pageSize)
      if (response.success && response.data) {
        set((state) => ({
          domainsByAccount: {
            ...state.domainsByAccount,
            [accountId]: {
              domains: response.data?.items ?? [],
              lastUpdated: Date.now(),
              page: response.data?.page ?? 1,
              hasMore: response.data?.hasMore ?? false,
            },
          },
        }))
        get().saveToStorage()
      } else {
        throw new Error(getErrorMessage(response.error))
      }
    } catch (err) {
      if (isCredentialError(err)) {
        useAccountStore.getState().fetchAccounts()
      }
      throw err
    } finally {
      set((state) => {
        const newSet = new Set(state.loadingAccounts)
        newSet.delete(accountId)
        return { loadingAccounts: newSet }
      })
    }
  },

  loadMoreDomains: async (accountId) => {
    const { loadingMoreAccounts, domainsByAccount } = get()
    const cache = domainsByAccount[accountId]

    if (!cache?.hasMore || loadingMoreAccounts.has(accountId)) return

    set((state) => ({
      loadingMoreAccounts: new Set(state.loadingMoreAccounts).add(accountId),
    }))

    const nextPage = cache.page + 1

    try {
      const pageSize = getDomainPageSize(accountId)
      const response = await domainService.listDomains(accountId, nextPage, pageSize)
      if (response.success && response.data) {
        set((state) => ({
          domainsByAccount: {
            ...state.domainsByAccount,
            [accountId]: {
              domains: [...cache.domains, ...(response.data?.items ?? [])],
              lastUpdated: Date.now(),
              page: response.data?.page ?? nextPage,
              hasMore: response.data?.hasMore ?? false,
            },
          },
        }))
        get().saveToStorage()
      }
    } catch (err) {
      logger.error("加载更多域名失败:", extractErrorMessage(err))
    } finally {
      set((state) => {
        const newSet = new Set(state.loadingMoreAccounts)
        newSet.delete(accountId)
        return { loadingMoreAccounts: newSet }
      })
    }
  },

  selectDomain: (accountId, domainId) => {
    set({ selectedAccountId: accountId, selectedDomainId: domainId })
  },

  clearAccountCache: (accountId) => {
    set((state) => {
      const { [accountId]: _, ...rest } = state.domainsByAccount
      return { domainsByAccount: rest }
    })
    get().saveToStorage()
  },

  clearAllCache: () => {
    set({ domainsByAccount: {} })
    storage.remove("domainsCache")
  },

  getDomainsForAccount: (accountId) => {
    return get().domainsByAccount[accountId]?.domains ?? []
  },

  getSelectedDomain: () => {
    const { selectedAccountId, selectedDomainId, domainsByAccount } = get()
    if (!(selectedAccountId && selectedDomainId)) return null
    const cache = domainsByAccount[selectedAccountId]
    return cache?.domains.find((d) => d.id === selectedDomainId) ?? null
  },

  isAccountLoading: (accountId) => {
    return get().loadingAccounts.has(accountId)
  },

  isAccountLoadingMore: (accountId) => {
    return get().loadingMoreAccounts.has(accountId)
  },

  hasMoreDomains: (accountId) => {
    return get().domainsByAccount[accountId]?.hasMore ?? false
  },

  // ===== Slices =====
  ...createDomainUiSlice(set, get),
  ...createDomainFilterSlice(set, get),
  ...createDomainMetadataSlice(set, get),
  ...createDomainBatchSlice(set, get),
}))
