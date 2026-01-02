import { create } from "zustand"
import { PAGINATION, STORAGE_KEYS, TIMING } from "@/constants"
import { extractErrorMessage, getErrorMessage, isCredentialError } from "@/lib/error"
import { logger } from "@/lib/logger"
import { domainMetadataService, domainService } from "@/services"
import type { Domain } from "@/types"
import { useAccountStore } from "./accountStore"

interface AccountDomainCache {
  domains: Domain[]
  lastUpdated: number
  page: number
  hasMore: boolean
}

// 收藏域名数据结构
export interface FavoriteDomain {
  accountId: string
  domainId: string
  domainName: string
  accountName: string
  provider: string
  favoritedAt: number
}

// 从 localStorage 读取初始缓存数据
function getInitialCache(): {
  domainsByAccount: Record<string, AccountDomainCache>
  scrollPosition: number
} {
  try {
    const cached = localStorage.getItem(STORAGE_KEYS.DOMAINS_CACHE)
    if (cached) {
      const parsed = JSON.parse(cached)
      // 兼容新旧格式
      if (parsed.domainsByAccount) {
        return {
          domainsByAccount: parsed.domainsByAccount,
          scrollPosition: parsed.scrollPosition ?? 0,
        }
      }
      // 旧格式
      return { domainsByAccount: parsed, scrollPosition: 0 }
    }
  } catch (err) {
    // 解析错误时清空缓存
    logger.warn("Failed to load domain cache, clearing:", err)
    localStorage.removeItem(STORAGE_KEYS.DOMAINS_CACHE)
  }
  return { domainsByAccount: {}, scrollPosition: 0 }
}

const initialCache = getInitialCache()

// 滚动位置保存的防抖 timer
let scrollSaveTimer: ReturnType<typeof setTimeout> | null = null

/**
 * 获取域名列表的分页大小
 *
 * 根据账户的 provider 限制动态计算，确保不超过 API 限制
 */
const getDomainPageSize = (accountId: string): number => {
  const { accounts, providers } = useAccountStore.getState()
  const account = accounts.find((a) => a.id === accountId)
  if (!account) {
    return PAGINATION.PAGE_SIZE
  }

  const provider = providers.find((p) => p.id === account.provider)
  const maxPageSize = provider?.limits.maxPageSizeDomains ?? 100

  return Math.min(PAGINATION.PAGE_SIZE, maxPageSize)
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

  // UI 状态（会话内保持，不持久化）
  expandedAccounts: Set<string>
  scrollPosition: number

  // 标签筛选状态（会话内保持，不持久化）
  selectedTags: Set<string>

  // 方法
  loadFromStorage: () => void
  saveToStorage: () => void
  refreshAllAccounts: (accounts: { id: string }[]) => Promise<void>
  refreshAccount: (accountId: string) => Promise<void>
  loadMoreDomains: (accountId: string) => Promise<void>
  selectDomain: (accountId: string, domainId: string) => void
  clearAccountCache: (accountId: string) => void
  clearAllCache: () => void

  // UI 状态方法
  toggleExpandedAccount: (accountId: string) => void
  setScrollPosition: (position: number) => void

  // 元数据操作
  toggleFavorite: (accountId: string, domainId: string) => Promise<void>
  getFavoriteDomains: () => FavoriteDomain[]

  // 标签操作
  addTag: (accountId: string, domainId: string, tag: string) => Promise<void>
  removeTag: (accountId: string, domainId: string, tag: string) => Promise<void>
  setTags: (accountId: string, domainId: string, tags: string[]) => Promise<void>

  // 标签筛选
  setSelectedTags: (tags: string[]) => void
  clearTagFilters: () => void
  getAllUsedTags: () => string[]

  // 便捷 getters
  getDomainsForAccount: (accountId: string) => Domain[]
  getSelectedDomain: () => Domain | null
  isAccountLoading: (accountId: string) => boolean
  isAccountLoadingMore: (accountId: string) => boolean
  hasMoreDomains: (accountId: string) => boolean
  isAccountExpanded: (accountId: string) => boolean
}

export const useDomainStore = create<DomainState>((set, get) => ({
  domainsByAccount: initialCache.domainsByAccount,
  selectedAccountId: null,
  selectedDomainId: null,
  loadingAccounts: new Set(),
  loadingMoreAccounts: new Set(),
  isBackgroundRefreshing: false,
  expandedAccounts: new Set(),
  scrollPosition: initialCache.scrollPosition,
  selectedTags: new Set(),

  // 从 localStorage 加载缓存
  loadFromStorage: () => {
    try {
      const cached = localStorage.getItem(STORAGE_KEYS.DOMAINS_CACHE)
      if (cached) {
        const parsed = JSON.parse(cached)
        // 兼容旧格式（直接是 domainsByAccount）和新格式（包含 scrollPosition）
        if (parsed.domainsByAccount) {
          set({
            domainsByAccount: parsed.domainsByAccount as Record<string, AccountDomainCache>,
            scrollPosition: parsed.scrollPosition ?? 0,
          })
        } else {
          // 旧格式
          set({ domainsByAccount: parsed as Record<string, AccountDomainCache> })
        }
      }
    } catch (err) {
      logger.error("Failed to load domain cache from storage:", err)
    }
  },

  // 保存到 localStorage
  saveToStorage: () => {
    try {
      const { domainsByAccount, scrollPosition } = get()
      localStorage.setItem(
        STORAGE_KEYS.DOMAINS_CACHE,
        JSON.stringify({ domainsByAccount, scrollPosition })
      )
    } catch (err) {
      logger.error("Failed to save domain cache to storage:", err)
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
            const pageSize = getDomainPageSize(account.id)
            const response = await domainService.listDomains(account.id, 1, pageSize)
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
      const pageSize = getDomainPageSize(accountId)
      const response = await domainService.listDomains(accountId, 1, pageSize)
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
      const pageSize = getDomainPageSize(accountId)
      const response = await domainService.listDomains(accountId, nextPage, pageSize)
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
      logger.error("加载更多域名失败:", extractErrorMessage(err))
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

  // 切换账户展开状态
  toggleExpandedAccount: (accountId) => {
    set((state) => {
      const next = new Set(state.expandedAccounts)
      if (next.has(accountId)) {
        next.delete(accountId)
      } else {
        next.add(accountId)
      }
      return { expandedAccounts: next }
    })
  },

  // 设置滚动位置（带防抖保存）
  setScrollPosition: (position) => {
    set({ scrollPosition: position })
    // 防抖：300ms 后才保存到 localStorage
    if (scrollSaveTimer) {
      clearTimeout(scrollSaveTimer)
    }
    scrollSaveTimer = setTimeout(() => {
      get().saveToStorage()
      scrollSaveTimer = null
    }, TIMING.SCROLL_SAVE_DEBOUNCE)
  },

  // 检查账户是否展开
  isAccountExpanded: (accountId) => {
    return get().expandedAccounts.has(accountId)
  },

  // 切换收藏状态
  toggleFavorite: async (accountId, domainId) => {
    const response = await domainMetadataService.toggleFavorite(accountId, domainId)

    if (!response.success || response.data === undefined) {
      logger.error("Failed to toggle favorite:", response.error)
      return
    }

    const newFavoriteState = response.data

    // 更新本地缓存
    set((state) => {
      const cache = state.domainsByAccount[accountId]
      if (!cache) return {}

      const domains = cache.domains.map((d) => {
        if (d.id === domainId) {
          // 保留原有的 favoritedAt（如果存在）
          const existingFavoritedAt = d.metadata?.favoritedAt

          return {
            ...d,
            metadata: {
              isFavorite: newFavoriteState,
              tags: d.metadata?.tags ?? [],
              updatedAt: new Date().toISOString(),
              // 首次收藏时记录时间，之后永远保留
              favoritedAt: existingFavoritedAt ?? new Date().toISOString(),
            },
          }
        }
        return d
      })

      return {
        domainsByAccount: {
          ...state.domainsByAccount,
          [accountId]: { ...cache, domains },
        },
      }
    })

    // 保存到 localStorage
    get().saveToStorage()
  },

  // 获取所有收藏域名（按收藏时间倒序）
  getFavoriteDomains: () => {
    const { domainsByAccount } = get()
    const { accounts } = useAccountStore.getState()

    const favorites: FavoriteDomain[] = []

    // 遍历所有账户的域名缓存
    Object.entries(domainsByAccount).forEach(([accountId, cache]) => {
      const account = accounts.find((a) => a.id === accountId)
      if (!(account && cache?.domains)) return

      // 过滤收藏域名
      cache.domains.forEach((domain) => {
        if (domain.metadata?.isFavorite) {
          favorites.push({
            accountId,
            domainId: domain.id,
            domainName: domain.name,
            accountName: account.name,
            provider: domain.provider,
            // 优先使用 favoritedAt，回退到 updatedAt（转换为时间戳）
            favoritedAt: new Date(
              domain.metadata.favoritedAt ?? domain.metadata.updatedAt
            ).getTime(),
          })
        }
      })
    })

    // 按收藏时间倒序排序（最新收藏在前）
    return favorites.sort((a, b) => b.favoritedAt - a.favoritedAt)
  },

  // 添加标签
  addTag: async (accountId, domainId, tag) => {
    const response = await domainMetadataService.addTag(accountId, domainId, tag)

    if (!(response.success && response.data)) {
      logger.error("Failed to add tag:", response.error)
      return
    }

    const newTags = response.data

    // 更新本地缓存
    set((state) => {
      const cache = state.domainsByAccount[accountId]
      if (!cache) return {}

      const domains = cache.domains.map((d) => {
        if (d.id === domainId) {
          return {
            ...d,
            metadata: {
              ...d.metadata,
              isFavorite: d.metadata?.isFavorite ?? false,
              tags: newTags,
              updatedAt: new Date().toISOString(),
            },
          }
        }
        return d
      })

      return {
        domainsByAccount: {
          ...state.domainsByAccount,
          [accountId]: { ...cache, domains },
        },
      }
    })

    get().saveToStorage()
  },

  // 移除标签
  removeTag: async (accountId, domainId, tag) => {
    const response = await domainMetadataService.removeTag(accountId, domainId, tag)

    if (!(response.success && response.data)) {
      logger.error("Failed to remove tag:", response.error)
      return
    }

    const newTags = response.data

    // 更新本地缓存
    set((state) => {
      const cache = state.domainsByAccount[accountId]
      if (!cache) return {}

      const domains = cache.domains.map((d) => {
        if (d.id === domainId) {
          return {
            ...d,
            metadata: {
              ...d.metadata,
              isFavorite: d.metadata?.isFavorite ?? false,
              tags: newTags,
              updatedAt: new Date().toISOString(),
            },
          }
        }
        return d
      })

      return {
        domainsByAccount: {
          ...state.domainsByAccount,
          [accountId]: { ...cache, domains },
        },
      }
    })

    get().saveToStorage()
  },

  // 批量设置标签
  setTags: async (accountId, domainId, tags) => {
    const response = await domainMetadataService.setTags(accountId, domainId, tags)

    if (!(response.success && response.data)) {
      logger.error("Failed to set tags:", response.error)
      return
    }

    const newTags = response.data

    // 更新本地缓存
    set((state) => {
      const cache = state.domainsByAccount[accountId]
      if (!cache) return {}

      const domains = cache.domains.map((d) => {
        if (d.id === domainId) {
          return {
            ...d,
            metadata: {
              ...d.metadata,
              isFavorite: d.metadata?.isFavorite ?? false,
              tags: newTags,
              updatedAt: new Date().toISOString(),
            },
          }
        }
        return d
      })

      return {
        domainsByAccount: {
          ...state.domainsByAccount,
          [accountId]: { ...cache, domains },
        },
      }
    })

    get().saveToStorage()
  },

  // 设置标签筛选
  setSelectedTags: (tags) => {
    set({ selectedTags: new Set(tags) })
  },

  // 清空标签筛选
  clearTagFilters: () => {
    set({ selectedTags: new Set() })
  },

  // 获取所有使用过的标签
  getAllUsedTags: () => {
    const { domainsByAccount } = get()
    const tagsSet = new Set<string>()

    Object.values(domainsByAccount).forEach((cache) => {
      cache.domains.forEach((domain) => {
        domain.metadata?.tags?.forEach((tag) => tagsSet.add(tag))
      })
    })

    return Array.from(tagsSet).sort()
  },
}))
