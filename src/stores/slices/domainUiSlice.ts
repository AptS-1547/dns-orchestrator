import { TIMING } from "@/constants"
import type { DomainGet, DomainSet } from "../domainStore.types"

let scrollSaveTimer: ReturnType<typeof setTimeout> | null = null

export function createDomainUiSlice(set: DomainSet, get: DomainGet) {
  return {
    toggleExpandedAccount: (accountId: string) => {
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

    setScrollPosition: (position: number) => {
      set({ scrollPosition: position })
      if (scrollSaveTimer) {
        clearTimeout(scrollSaveTimer)
      }
      scrollSaveTimer = setTimeout(() => {
        get().saveToStorage()
        scrollSaveTimer = null
      }, TIMING.SCROLL_SAVE_DEBOUNCE)
    },

    isAccountExpanded: (accountId: string) => {
      return get().expandedAccounts.has(accountId)
    },
  }
}
