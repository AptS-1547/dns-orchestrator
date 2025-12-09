import { create } from "zustand"
import { PAGINATION, STORAGE_KEYS } from "@/constants"
import { extractErrorMessage, getErrorMessage, isCredentialError } from "@/lib/error"
import { domainService } from "@/services"
import type { Domain } from "@/types"
import { useAccountStore } from "./accountStore"

interface AccountDomainCache {
  domains: Domain[]
  lastUpdated: number
  page: number
  hasMore: boolean
}

interface DomainState {
  // 按账户分组的域名缓存
  domainsByAccount: Record<string, AccountDomainCache>

  // 当前选中
  selectedAccountId: string | null
  selectedDomainId: string | null

  // 加载状态（按账户）
  loadingAccounts: Set<string>
  loadingMoreAccounts: Set<string>

  // 后台刷新状态
  isBackgroundRefreshing: boolean

  // 方法
  loadFromStorage: () => void
  saveToStorage: () => void
  refreshAllAccounts: (accounts: { id: string }[]) => Promise<void>
  refreshAccount: (accountId: string) => Promise<void>
  loadMoreDomains: (accountId: string) => Promise<void>
  selectDomain: (accountId: string, domainId: string) => void
  clearAccountCache: (accountId: string) => void
  clearAllCache: () => void

  // 便捷 getters
  getDomainsForAccount: (accountId: string) => Domain[]
  getSelectedDomain: () => Domain | null
  isAccountLoading: (accountId: string) => boolean
  isAccountLoadingMore: (accountId: string) => boolean
  hasMoreDomains: (accountId: string) => boolean
}

export const useDomainStore = create<DomainState>((set, get) => ({
  domainsByAccount: {},
  selectedAccountId: null,
  selectedDomainId: null,
  loadingAccounts: new Set(),
  loadingMoreAccounts: new Set(),
  isBackgroundRefreshing: false,

  // 从 localStorage 加载缓存
  loadFromStorage: () => {
    try {
      const cached = localStorage.getItem(STORAGE_KEYS.DOMAINS_CACHE)
      if (cached) {
        const parsed = JSON.parse(cached) as Record<string, AccountDomainCache>
        set({ domainsByAccount: parsed })
      }
    } catch (err) {
      console.error("Failed to load domain cache from storage:", err)
    }
  },

  // 保存到 localStorage
  saveToStorage: () => {
    try {
      const { domainsByAccount } = get()
      localStorage.setItem(STORAGE_KEYS.DOMAINS_CACHE, JSON.stringify(domainsByAccount))
    } catch (err) {
      console.error("Failed to save domain cache to storage:", err)
    }
  },

  // 后台刷新所有账户
  refreshAllAccounts: async (accounts) => {
    const { isBackgroundRefreshing } = get()
    if (isBackgroundRefreshing) return

    set({ isBackgroundRefreshing: true })

    try {
      // 并行刷新所有账户（静默失败）
      await Promise.allSettled(
        accounts.map(async (account) => {
          try {
            const response = await domainService.listDomains(account.id, 1, PAGINATION.PAGE_SIZE)
            if (response.success && response.data) {
              set((state) => ({
                domainsByAccount: {
                  ...state.domainsByAccount,
                  [account.id]: {
                    domains: response.data!.items,
                    lastUpdated: Date.now(),
                    page: response.data!.page,
                    hasMore: response.data!.hasMore,
                  },
                },
              }))
            }
          } catch {
            // 静默失败，不影响其他账户
          }
        })
      )
      // 保存到 localStorage
      get().saveToStorage()
    } finally {
      set({ isBackgroundRefreshing: false })
    }
  },

  // 刷新单个账户
  refreshAccount: async (accountId) => {
    const { loadingAccounts } = get()
    if (loadingAccounts.has(accountId)) return

    // 添加到加载中
    set((state) => ({
      loadingAccounts: new Set(state.loadingAccounts).add(accountId),
    }))

    try {
      const response = await domainService.listDomains(accountId, 1, PAGINATION.PAGE_SIZE)
      if (response.success && response.data) {
        set((state) => ({
          domainsByAccount: {
            ...state.domainsByAccount,
            [accountId]: {
              domains: response.data!.items,
              lastUpdated: Date.now(),
              page: response.data!.page,
              hasMore: response.data!.hasMore,
            },
          },
        }))
        get().saveToStorage()
      } else {
        throw new Error(getErrorMessage(response.error))
      }
    } catch (err) {
      // 凭证错误时刷新账户列表
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

  // 加载更多域名
  loadMoreDomains: async (accountId) => {
    const { loadingMoreAccounts, domainsByAccount } = get()
    const cache = domainsByAccount[accountId]

    if (!cache?.hasMore || loadingMoreAccounts.has(accountId)) return

    set((state) => ({
      loadingMoreAccounts: new Set(state.loadingMoreAccounts).add(accountId),
    }))

    const nextPage = cache.page + 1

    try {
      const response = await domainService.listDomains(accountId, nextPage, PAGINATION.PAGE_SIZE)
      if (response.success && response.data) {
        set((state) => ({
          domainsByAccount: {
            ...state.domainsByAccount,
            [accountId]: {
              domains: [...cache.domains, ...response.data!.items],
              lastUpdated: Date.now(),
              page: response.data!.page,
              hasMore: response.data!.hasMore,
            },
          },
        }))
        get().saveToStorage()
      }
    } catch (err) {
      console.error("加载更多域名失败:", extractErrorMessage(err))
    } finally {
      set((state) => {
        const newSet = new Set(state.loadingMoreAccounts)
        newSet.delete(accountId)
        return { loadingMoreAccounts: newSet }
      })
    }
  },

  // 选择域名
  selectDomain: (accountId, domainId) => {
    set({ selectedAccountId: accountId, selectedDomainId: domainId })
  },

  // 清理单个账户的缓存
  clearAccountCache: (accountId) => {
    set((state) => {
      const { [accountId]: _, ...rest } = state.domainsByAccount
      return { domainsByAccount: rest }
    })
    get().saveToStorage()
  },

  // 清理所有缓存
  clearAllCache: () => {
    set({ domainsByAccount: {} })
    localStorage.removeItem(STORAGE_KEYS.DOMAINS_CACHE)
  },

  // 获取账户的域名列表
  getDomainsForAccount: (accountId) => {
    return get().domainsByAccount[accountId]?.domains ?? []
  },

  // 获取当前选中的域名
  getSelectedDomain: () => {
    const { selectedAccountId, selectedDomainId, domainsByAccount } = get()
    if (!(selectedAccountId && selectedDomainId)) return null
    const cache = domainsByAccount[selectedAccountId]
    return cache?.domains.find((d) => d.id === selectedDomainId) ?? null
  },

  // 检查账户是否正在加载
  isAccountLoading: (accountId) => {
    return get().loadingAccounts.has(accountId)
  },

  // 检查账户是否正在加载更多
  isAccountLoadingMore: (accountId) => {
    return get().loadingMoreAccounts.has(accountId)
  },

  // 检查是否有更多域名
  hasMoreDomains: (accountId) => {
    return get().domainsByAccount[accountId]?.hasMore ?? false
  },
}))
