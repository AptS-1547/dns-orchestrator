import i18n from "@/i18n"
import { extractErrorMessage, getErrorMessage } from "@/lib/error"
import type { ApiResponse, BatchTagRequest, BatchTagResult, Domain, DomainMetadata } from "@/types"
import { toast } from "sonner"

/**
 * 账户域名缓存
 */
export interface AccountDomainCache {
  domains: Domain[]
  lastUpdated: number
  page: number
  hasMore: boolean
}

/**
 * 收藏域名数据
 */
export interface FavoriteDomain {
  accountId: string
  domainId: string
  domainName: string
  accountName: string
  provider: string
  favoritedAt: number
}

/**
 * Domain Store 完整状态类型
 */
export interface DomainState {
  domainsByAccount: Record<string, AccountDomainCache>
  selectedAccountId: string | null
  selectedDomainId: string | null
  loadingAccounts: Set<string>
  loadingMoreAccounts: Set<string>
  isBackgroundRefreshing: boolean
  expandedAccounts: Set<string>
  scrollPosition: number
  selectedTags: Set<string>
  selectedDomainKeys: Set<string>
  isBatchMode: boolean
  isBatchOperating: boolean

  loadFromStorage: () => void
  saveToStorage: () => void
  refreshAllAccounts: (accounts: { id: string }[]) => Promise<void>
  refreshAccount: (accountId: string) => Promise<void>
  loadMoreDomains: (accountId: string) => Promise<void>
  selectDomain: (accountId: string, domainId: string) => void
  clearAccountCache: (accountId: string) => void
  clearAllCache: () => void

  toggleExpandedAccount: (accountId: string) => void
  setScrollPosition: (position: number) => void
  isAccountExpanded: (accountId: string) => boolean

  toggleFavorite: (accountId: string, domainId: string) => Promise<void>
  getFavoriteDomains: () => FavoriteDomain[]
  addTag: (accountId: string, domainId: string, tag: string) => Promise<void>
  removeTag: (accountId: string, domainId: string, tag: string) => Promise<void>
  setTags: (accountId: string, domainId: string, tags: string[]) => Promise<void>
  updateMetadata: (
    accountId: string,
    domainId: string,
    update: import("@/types").DomainMetadataUpdate
  ) => Promise<void>
  setColor: (accountId: string, domainId: string, color: string | null) => Promise<void>
  setNote: (accountId: string, domainId: string, note: string | null) => Promise<void>

  setSelectedTags: (tags: string[]) => void
  clearTagFilters: () => void
  getAllUsedTags: () => string[]

  toggleBatchMode: () => void
  toggleDomainSelection: (accountId: string, domainId: string) => void
  selectAllDomains: (accountId: string) => void
  clearDomainSelection: () => void
  batchAddTags: (tags: string[]) => Promise<void>
  batchRemoveTags: (tags: string[]) => Promise<void>
  batchSetTags: (tags: string[]) => Promise<void>

  getDomainsForAccount: (accountId: string) => Domain[]
  getSelectedDomain: () => Domain | null
  isAccountLoading: (accountId: string) => boolean
  isAccountLoadingMore: (accountId: string) => boolean
  hasMoreDomains: (accountId: string) => boolean
}

export type DomainSet = (
  partial: Partial<DomainState> | ((state: DomainState) => Partial<DomainState>)
) => void
export type DomainGet = () => DomainState

/**
 * 构造包含标签更新的元数据对象
 */
export function buildMetadataWithTags(domain: Domain, newTags: string[]): DomainMetadata {
  return {
    ...domain.metadata,
    isFavorite: domain.metadata?.isFavorite ?? false,
    tags: newTags,
    color: domain.metadata?.color || "none",
    updatedAt: new Date().toISOString(),
  }
}

/**
 * 在账户缓存中更新单个域名的元数据
 */
export function updateDomainInCache(
  state: DomainState,
  accountId: string,
  domainId: string,
  metadataBuilder: (domain: Domain) => DomainMetadata
): Partial<DomainState> {
  const cache = state.domainsByAccount[accountId]
  if (!cache) return {}

  const domains = cache.domains.map((d) => {
    if (d.id === domainId) {
      return { ...d, metadata: metadataBuilder(d) }
    }
    return d
  })

  return {
    domainsByAccount: {
      ...state.domainsByAccount,
      [accountId]: { ...cache, domains },
    },
  }
}

/**
 * 批量标签操作的本地更新逻辑
 */
export type BatchTagLocalUpdater = (existingTags: string[], tagsToApply: string[]) => string[]

/**
 * 执行批量标签操作的通用逻辑
 */
export async function executeBatchTagOperation(
  selectedDomainKeys: Set<string>,
  tags: string[],
  apiCall: (requests: BatchTagRequest[]) => Promise<ApiResponse<BatchTagResult>>,
  localUpdater: BatchTagLocalUpdater,
  successMessage: string,
  partialMessage: string,
  set: DomainSet,
  get: DomainGet
): Promise<void> {
  if (selectedDomainKeys.size === 0) return

  set({ isBatchOperating: true })
  try {
    const requests = Array.from(selectedDomainKeys).map((key) => {
      const [accountId, domainId] = key.split("::")
      return { accountId, domainId, tags }
    })

    const response = await apiCall(requests)

    if (response.success && response.data) {
      const result = response.data

      const successKeys = new Set(
        requests
          .filter(
            (req) =>
              !result.failures.some(
                (f) => f.accountId === req.accountId && f.domainId === req.domainId
              )
          )
          .map((req) => `${req.accountId}::${req.domainId}`)
      )

      set((state) => {
        const newDomainsByAccount = { ...state.domainsByAccount }

        successKeys.forEach((key) => {
          const [accountId, domainId] = key.split("::")
          const cache = newDomainsByAccount[accountId]
          if (!cache) return

          newDomainsByAccount[accountId] = {
            ...cache,
            domains: cache.domains.map((d) => {
              if (d.id === domainId) {
                const existingTags = d.metadata?.tags ?? []
                const newTags = localUpdater(existingTags, tags)
                return { ...d, metadata: buildMetadataWithTags(d, newTags) }
              }
              return d
            }),
          }
        })

        return {
          domainsByAccount: newDomainsByAccount,
          selectedDomainKeys: new Set(),
          isBatchMode: false,
        }
      })

      get().saveToStorage()

      if (result.failedCount === 0) {
        toast.success(i18n.t(successMessage, { count: result.successCount }))
      } else {
        toast.warning(
          i18n.t(partialMessage, {
            success: result.successCount,
            failed: result.failedCount,
          })
        )
      }
    } else {
      toast.error(getErrorMessage(response.error))
    }
  } catch (err) {
    toast.error(extractErrorMessage(err))
  } finally {
    set({ isBatchOperating: false })
  }
}
