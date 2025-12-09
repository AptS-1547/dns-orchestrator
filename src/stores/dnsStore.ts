import { toast } from "sonner"
import { create } from "zustand"
import { PAGINATION } from "@/constants"
import { extractErrorMessage, getErrorMessage } from "@/lib/error"
import { invoke } from "@/lib/tauri"
import type {
  BatchDeleteRequest,
  BatchDeleteResult,
  CreateDnsRecordRequest,
  DnsRecord,
  UpdateDnsRecordRequest,
} from "@/types"

interface DnsState {
  records: DnsRecord[]
  currentDomainId: string | null
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  error: string | null
  // 分页状态
  page: number
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
  hasMore: false,
  totalCount: 0,
  keyword: "",
  recordType: "",
  selectedRecordIds: new Set(),
  isSelectMode: false,
  isBatchDeleting: false,

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
      const response = await invoke("list_dns_records", {
        accountId,
        domainId,
        page: 1,
        pageSize: PAGINATION.PAGE_SIZE,
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
      const response = await invoke("list_dns_records", {
        accountId,
        domainId,
        page: nextPage,
        pageSize: PAGINATION.PAGE_SIZE,
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
      console.error("加载更多记录失败:", err)
    } finally {
      if (get().currentDomainId === domainId) {
        set({ isLoadingMore: false })
      }
    }
  },

  createRecord: async (accountId, request) => {
    set({ isLoading: true, error: null })
    try {
      const response = await invoke("create_dns_record", { accountId, request })
      if (response.success && response.data) {
        set((state) => ({
          records: [...state.records, response.data!],
          totalCount: state.totalCount + 1,
        }))
        toast.success(`记录 "${response.data.name}" 添加成功`)
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
    try {
      const response = await invoke("update_dns_record", { accountId, recordId, request })
      if (response.success && response.data) {
        set((state) => ({
          records: state.records.map((r) => (r.id === recordId ? response.data! : r)),
        }))
        toast.success("记录更新成功")
        return true
      }
      toast.error("更新记录失败")
      return false
    } catch (err) {
      toast.error(extractErrorMessage(err))
      return false
    }
  },

  deleteRecord: async (accountId, recordId, domainId) => {
    set({ isDeleting: true })
    try {
      const response = await invoke("delete_dns_record", { accountId, recordId, domainId })
      if (response.success) {
        set((state) => ({
          records: state.records.filter((r) => r.id !== recordId),
          totalCount: Math.max(0, state.totalCount - 1),
        }))
        toast.success("记录已删除")
        return true
      }
      toast.error("删除记录失败")
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
      const response = await invoke("batch_delete_dns_records", { accountId, request })

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
          toast.success(`成功删除 ${result.successCount} 条记录`)
        } else {
          toast.warning(`成功删除 ${result.successCount} 条记录，${result.failedCount} 条失败`)
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
