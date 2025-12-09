import { create } from "zustand"
import { PAGINATION } from "@/constants"
import { extractErrorMessage, getErrorMessage, isCredentialError } from "@/lib/error"
import { invoke } from "@/lib/tauri"
import type { Domain } from "@/types"
import { useAccountStore } from "./accountStore"

interface DomainState {
  domains: Domain[]
  selectedDomainId: string | null
  isLoading: boolean
  isLoadingMore: boolean
  error: string | null
  // 分页状态
  page: number
  hasMore: boolean
  totalCount: number
  currentAccountId: string | null

  fetchDomains: (accountId: string) => Promise<void>
  fetchMoreDomains: (accountId: string) => Promise<void>
  selectDomain: (id: string | null) => void
  clearDomains: () => void
}

export const useDomainStore = create<DomainState>((set, get) => ({
  domains: [],
  selectedDomainId: null,
  isLoading: false,
  isLoadingMore: false,
  error: null,
  page: 1,
  hasMore: false,
  totalCount: 0,
  currentAccountId: null,

  fetchDomains: async (accountId) => {
    const { currentAccountId, selectedDomainId } = get()
    const isAccountChanged = currentAccountId !== accountId

    set({
      isLoading: true,
      error: null,
      selectedDomainId: isAccountChanged ? null : selectedDomainId,
      domains: [],
      page: 1,
      hasMore: false,
      totalCount: 0,
      currentAccountId: accountId,
    })
    try {
      const response = await invoke("list_domains", {
        accountId,
        page: 1,
        pageSize: PAGINATION.PAGE_SIZE,
      })
      // 验证请求是否仍然有效
      if (get().currentAccountId !== accountId) {
        return
      }
      if (response.success && response.data) {
        set({
          domains: response.data.items,
          page: response.data.page,
          hasMore: response.data.hasMore,
          totalCount: response.data.totalCount,
        })
      } else {
        set({ error: getErrorMessage(response.error) })
      }
    } catch (err) {
      if (get().currentAccountId !== accountId) {
        return
      }
      // 凭证错误时刷新账户列表以获取最新状态
      if (isCredentialError(err)) {
        useAccountStore.getState().fetchAccounts()
      }
      set({ error: extractErrorMessage(err) })
    } finally {
      if (get().currentAccountId === accountId) {
        set({ isLoading: false })
      }
    }
  },

  fetchMoreDomains: async (accountId) => {
    const { isLoadingMore, hasMore, page, currentAccountId, domains } = get()
    if (isLoadingMore || !hasMore || currentAccountId !== accountId) {
      return
    }

    set({ isLoadingMore: true })
    const nextPage = page + 1

    try {
      const response = await invoke("list_domains", {
        accountId,
        page: nextPage,
        pageSize: PAGINATION.PAGE_SIZE,
      })
      // 验证请求是否仍然有效
      if (get().currentAccountId !== accountId) {
        return
      }
      if (response.success && response.data) {
        set({
          domains: [...domains, ...response.data.items],
          page: response.data.page,
          hasMore: response.data.hasMore,
        })
      }
    } catch (err) {
      console.error("加载更多域名失败:", err)
    } finally {
      if (get().currentAccountId === accountId) {
        set({ isLoadingMore: false })
      }
    }
  },

  selectDomain: (id) => set({ selectedDomainId: id }),

  clearDomains: () =>
    set({
      domains: [],
      selectedDomainId: null,
      page: 1,
      hasMore: false,
      totalCount: 0,
      currentAccountId: null,
    }),
}))
