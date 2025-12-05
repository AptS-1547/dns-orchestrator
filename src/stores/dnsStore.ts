import type {
  ApiResponse,
  CreateDnsRecordRequest,
  DnsRecord,
  PaginatedResponse,
  UpdateDnsRecordRequest,
} from "@/types"
import { invoke } from "@tauri-apps/api/core"
import { toast } from "sonner"
import { create } from "zustand"

const PAGE_SIZE = 20

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

  fetchRecords: (
    accountId: string,
    domainId: string,
    keyword?: string,
    recordType?: string
  ) => Promise<void>
  fetchMoreRecords: (accountId: string, domainId: string) => Promise<void>
  setSearchParams: (keyword: string, recordType: string) => void
  createRecord: (accountId: string, request: CreateDnsRecordRequest) => Promise<DnsRecord | null>
  updateRecord: (
    accountId: string,
    recordId: string,
    request: UpdateDnsRecordRequest
  ) => Promise<boolean>
  deleteRecord: (accountId: string, recordId: string, domainId: string) => Promise<boolean>
  clearRecords: () => void
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

  setSearchParams: (keyword, recordType) => {
    set({ keyword, recordType })
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
      const response = await invoke<ApiResponse<PaginatedResponse<DnsRecord>>>("list_dns_records", {
        accountId,
        domainId,
        page: 1,
        pageSize: PAGE_SIZE,
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
        const msg = response.error?.message || "获取 DNS 记录失败"
        set({ error: msg })
        toast.error(msg)
      }
    } catch (err) {
      if (get().currentDomainId !== domainId) {
        return // 请求已过期，忽略
      }
      const msg = String(err)
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
      const response = await invoke<ApiResponse<PaginatedResponse<DnsRecord>>>("list_dns_records", {
        accountId,
        domainId,
        page: nextPage,
        pageSize: PAGE_SIZE,
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
      const response = await invoke<ApiResponse<DnsRecord>>("create_dns_record", {
        accountId,
        request,
      })
      if (response.success && response.data) {
        set((state) => ({
          records: [...state.records, response.data!],
          totalCount: state.totalCount + 1,
        }))
        toast.success(`记录 "${response.data.name}" 添加成功`)
        return response.data
      }
      const msg = response.error?.message || "创建记录失败"
      set({ error: msg })
      toast.error(msg)
      return null
    } catch (err) {
      const msg = String(err)
      set({ error: msg })
      toast.error(msg)
      return null
    } finally {
      set({ isLoading: false })
    }
  },

  updateRecord: async (accountId, recordId, request) => {
    try {
      const response = await invoke<ApiResponse<DnsRecord>>("update_dns_record", {
        accountId,
        recordId,
        request,
      })
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
      toast.error(String(err))
      return false
    }
  },

  deleteRecord: async (accountId, recordId, domainId) => {
    set({ isDeleting: true })
    try {
      const response = await invoke<ApiResponse<void>>("delete_dns_record", {
        accountId,
        recordId,
        domainId,
      })
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
      toast.error(String(err))
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
    }),
}))
