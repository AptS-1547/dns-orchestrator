import { toast } from "sonner"
import { create } from "zustand"
import { PAGINATION } from "@/constants"
import i18n from "@/i18n"
import { extractErrorMessage, getErrorMessage } from "@/lib/error"
import { logger } from "@/lib/logger"
import { dnsService } from "@/services"
import { useAccountStore } from "@/stores/accountStore"
import type {
  BatchDeleteRequest,
  BatchDeleteResult,
  CreateDnsRecordRequest,
  DnsRecord,
  UpdateDnsRecordRequest,
} from "@/types"

/**
 * 获取 DNS 记录的分页大小
 *
 * 根据账户的 provider 限制动态计算，确保不超过 API 限制
 */
const getRecordPageSize = (accountId: string, preferredSize: number): number => {
  const { accounts, providers } = useAccountStore.getState()
  const account = accounts.find((a) => a.id === accountId)
  if (!account) {
    return preferredSize
  }

  const provider = providers.find((p) => p.id === account.provider)
  const maxPageSize = provider?.limits.maxPageSizeRecords ?? 100

  return Math.min(preferredSize, maxPageSize)
}

interface DnsState {
  records: DnsRecord[]
  currentDomainId: string | null
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  error: string | null
  // 分页状态
  page: number
  pageSize: number
  hasMore: boolean
  totalCount: number
  // 搜索状态
  keyword: string
  recordType: string
  // 批量选择状态
  selectedRecordIds: Set<string>
  isSelectMode: boolean
  isBatchDeleting: boolean

  fetchRecords: (
    accountId: string,
    domainId: string,
    keyword?: string,
    recordType?: string
  ) => Promise<void>
  fetchMoreRecords: (accountId: string, domainId: string) => Promise<void>
  jumpToPage: (accountId: string, domainId: string, targetPage: number) => Promise<void>
  setPageSize: (accountId: string, domainId: string, size: number) => Promise<void>
  setKeyword: (keyword: string) => void
  setRecordType: (recordType: string) => void
  createRecord: (accountId: string, request: CreateDnsRecordRequest) => Promise<DnsRecord | null>
  updateRecord: (
    accountId: string,
    recordId: string,
    request: UpdateDnsRecordRequest
  ) => Promise<boolean>
  deleteRecord: (accountId: string, recordId: string, domainId: string) => Promise<boolean>
  clearRecords: () => void
  // 批量选择方法
  toggleSelectMode: () => void
  toggleRecordSelection: (recordId: string) => void
  selectAllRecords: () => void
  clearSelection: () => void
  batchDeleteRecords: (accountId: string, domainId: string) => Promise<BatchDeleteResult | null>
}

export const useDnsStore = create<DnsState>((set, get) => ({
  records: [],
  currentDomainId: null,
  isLoading: false,
  isLoadingMore: false,
  isDeleting: false,
  error: null,
  page: 1,
  pageSize: PAGINATION.PAGE_SIZE,
  hasMore: false,
  totalCount: 0,
  keyword: "",
  recordType: "",
  selectedRecordIds: new Set(),
  isSelectMode: false,
  isBatchDeleting: false,

  setPageSize: async (accountId, domainId, size) => {
    set({ pageSize: size, page: 1 })
    // 重新加载第一页数据
    await get().fetchRecords(accountId, domainId)
  },

  setKeyword: (keyword) => {
    set({ keyword })
  },

  setRecordType: (recordType) => {
    set({ recordType })
  },

  fetchRecords: async (accountId, domainId, keyword, recordType) => {
    const { currentDomainId: prevDomainId } = get()
    const isDomainChange = prevDomainId !== domainId

    // 如果传入了 keyword/recordType，更新状态；否则使用当前状态
    const searchKeyword = keyword !== undefined ? keyword : get().keyword
    const searchRecordType = recordType !== undefined ? recordType : get().recordType

    set({
      isLoading: true,
      error: null,
      currentDomainId: domainId,
      page: 1,
      hasMore: false,
      keyword: searchKeyword,
      recordType: searchRecordType,
      // 只有切换 domain 时才清空 records，搜索时保持旧数据
      ...(isDomainChange && { records: [], totalCount: 0 }),
    })
    try {
      const pageSize = getRecordPageSize(accountId, get().pageSize)
      const response = await dnsService.listRecords({
        accountId,
        domainId,
        page: 1,
        pageSize,
        keyword: searchKeyword || null,
        recordType: searchRecordType || null,
      })
      // 只有当 domainId 匹配当前选中的域名时才更新
      if (get().currentDomainId !== domainId) {
        return // 请求已过期，忽略
      }
      if (response.success && response.data) {
        set({
          records: response.data.items,
          page: response.data.page,
          hasMore: response.data.hasMore,
          totalCount: response.data.totalCount,
        })
      } else {
        const msg = getErrorMessage(response.error)
        set({ error: msg })
        toast.error(msg)
      }
    } catch (err) {
      if (get().currentDomainId !== domainId) {
        return // 请求已过期，忽略
      }
      const msg = extractErrorMessage(err)
      set({ error: msg })
      toast.error(msg)
    } finally {
      if (get().currentDomainId === domainId) {
        set({ isLoading: false })
      }
    }
  },

  fetchMoreRecords: async (accountId, domainId) => {
    const { isLoadingMore, hasMore, page, currentDomainId, records, keyword, recordType } = get()
    if (isLoadingMore || !hasMore || currentDomainId !== domainId) {
      return
    }

    set({ isLoadingMore: true })
    const nextPage = page + 1

    try {
      const pageSize = getRecordPageSize(accountId, get().pageSize)
      const response = await dnsService.listRecords({
        accountId,
        domainId,
        page: nextPage,
        pageSize,
        keyword: keyword || null,
        recordType: recordType || null,
      })
      // 验证请求是否仍然有效
      if (get().currentDomainId !== domainId) {
        return
      }
      if (response.success && response.data) {
        set({
          records: [...records, ...response.data.items],
          page: response.data.page,
          hasMore: response.data.hasMore,
        })
      }
    } catch (err) {
      logger.error("加载更多记录失败:", err)
    } finally {
      if (get().currentDomainId === domainId) {
        set({ isLoadingMore: false })
      }
    }
  },

  jumpToPage: async (accountId, domainId, targetPage) => {
    const { keyword, recordType, totalCount, pageSize } = get()

    // 验证页码有效性
    const actualPageSize = getRecordPageSize(accountId, pageSize)
    const maxPage = Math.ceil(totalCount / actualPageSize)

    if (targetPage < 1 || targetPage > maxPage) {
      toast.error(`页码必须在 1-${maxPage} 之间`)
      return
    }

    // 设置加载状态并清空当前记录
    set({
      isLoading: true,
      error: null,
      records: [],
      page: targetPage,
    })

    try {
      const response = await dnsService.listRecords({
        accountId,
        domainId,
        page: targetPage,
        pageSize: actualPageSize,
        keyword: keyword || null,
        recordType: recordType || null,
      })

      // 验证请求是否仍然有效
      if (get().currentDomainId !== domainId) {
        return
      }

      if (response.success && response.data) {
        set({
          records: response.data.items,
          page: response.data.page,
          hasMore: response.data.hasMore,
          totalCount: response.data.totalCount,
        })
      } else {
        const msg = getErrorMessage(response.error)
        set({ error: msg })
        toast.error(msg)
      }
    } catch (err) {
      if (get().currentDomainId !== domainId) {
        return
      }
      const msg = extractErrorMessage(err)
      set({ error: msg })
      toast.error(msg)
    } finally {
      if (get().currentDomainId === domainId) {
        set({ isLoading: false })
      }
    }
  },

  createRecord: async (accountId, request) => {
    set({ isLoading: true, error: null })
    try {
      const response = await dnsService.createRecord(accountId, request)
      if (response.success && response.data) {
        set((state) => ({
          records: [...state.records, response.data!],
          totalCount: state.totalCount + 1,
        }))
        toast.success(i18n.t("dns.createSuccess", { name: response.data.name }))
        return response.data
      }
      const msg = getErrorMessage(response.error)
      set({ error: msg })
      toast.error(msg)
      return null
    } catch (err) {
      const msg = extractErrorMessage(err)
      set({ error: msg })
      toast.error(msg)
      return null
    } finally {
      set({ isLoading: false })
    }
  },

  updateRecord: async (accountId, recordId, request) => {
    set({ isLoading: true, error: null })
    try {
      const response = await dnsService.updateRecord(accountId, recordId, request)
      if (response.success && response.data) {
        set((state) => ({
          records: state.records.map((r) => (r.id === recordId ? response.data! : r)),
        }))
        toast.success(i18n.t("dns.updateSuccess"))
        return true
      }
      toast.error(i18n.t("dns.updateFailed"))
      return false
    } catch (err) {
      toast.error(extractErrorMessage(err))
      return false
    } finally {
      set({ isLoading: false })
    }
  },

  deleteRecord: async (accountId, recordId, domainId) => {
    set({ isDeleting: true })
    try {
      const response = await dnsService.deleteRecord(accountId, recordId, domainId)
      if (response.success) {
        set((state) => ({
          records: state.records.filter((r) => r.id !== recordId),
          totalCount: Math.max(0, state.totalCount - 1),
        }))
        toast.success(i18n.t("dns.deleteSuccess"))
        return true
      }
      toast.error(i18n.t("dns.deleteFailed"))
      return false
    } catch (err) {
      toast.error(extractErrorMessage(err))
      return false
    } finally {
      set({ isDeleting: false })
    }
  },

  clearRecords: () =>
    set({
      records: [],
      error: null,
      page: 1,
      hasMore: false,
      totalCount: 0,
      keyword: "",
      recordType: "",
      selectedRecordIds: new Set(),
      isSelectMode: false,
    }),

  toggleSelectMode: () => {
    const { isSelectMode } = get()
    set({
      isSelectMode: !isSelectMode,
      selectedRecordIds: new Set(), // 切换时清空选择
    })
  },

  toggleRecordSelection: (recordId) => {
    const { selectedRecordIds } = get()
    const newSet = new Set(selectedRecordIds)
    if (newSet.has(recordId)) {
      newSet.delete(recordId)
    } else {
      newSet.add(recordId)
    }
    set({ selectedRecordIds: newSet })
  },

  selectAllRecords: () => {
    const { records } = get()
    set({ selectedRecordIds: new Set(records.map((r) => r.id)) })
  },

  clearSelection: () => {
    set({ selectedRecordIds: new Set() })
  },

  batchDeleteRecords: async (accountId, domainId) => {
    const { selectedRecordIds } = get()
    if (selectedRecordIds.size === 0) return null

    set({ isBatchDeleting: true })
    try {
      const request: BatchDeleteRequest = {
        domainId,
        recordIds: Array.from(selectedRecordIds),
      }
      const response = await dnsService.batchDeleteRecords(accountId, request)

      if (response.success && response.data) {
        const result = response.data
        // 从列表中移除成功删除的记录
        const deletedIds = new Set(
          request.recordIds.filter((id) => !result.failures.some((f) => f.recordId === id))
        )
        set((state) => ({
          records: state.records.filter((r) => !deletedIds.has(r.id)),
          totalCount: Math.max(0, state.totalCount - result.successCount),
          selectedRecordIds: new Set(),
          isSelectMode: false,
        }))

        if (result.failedCount === 0) {
          toast.success(i18n.t("dns.batchDeleteSuccess", { count: result.successCount }))
        } else {
          toast.warning(
            i18n.t("dns.batchDeletePartial", {
              success: result.successCount,
              failed: result.failedCount,
            })
          )
        }
        return result
      }
      toast.error(getErrorMessage(response.error))
      return null
    } catch (err) {
      toast.error(extractErrorMessage(err))
      return null
    } finally {
      set({ isBatchDeleting: false })
    }
  },
}))
