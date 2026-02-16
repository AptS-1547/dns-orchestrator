import type { DomainGet, DomainSet } from "../domainStore.types"

export function createDomainFilterSlice(set: DomainSet, get: DomainGet) {
  return {
    setSelectedTags: (tags: string[]) => {
      set({ selectedTags: new Set(tags) })
    },

    clearTagFilters: () => {
      set({ selectedTags: new Set() })
    },

    getAllUsedTags: () => {
      const { domainsByAccount } = get()
      const tagsSet = new Set<string>()

      Object.values(domainsByAccount).forEach((cache) => {
        cache.domains.forEach((domain) => {
          domain.metadata?.tags?.forEach((tag) => {
            tagsSet.add(tag)
          })
        })
      })

      return Array.from(tagsSet).sort()
    },
  }
}
