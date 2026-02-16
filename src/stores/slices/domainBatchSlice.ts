import { domainMetadataService } from "@/services"
import {
  type DomainGet,
  type DomainSet,
  executeBatchTagOperation,
} from "../domainStore.types"

export function createDomainBatchSlice(set: DomainSet, get: DomainGet) {
  const mergeTagsUpdater = (existingTags: string[], tagsToAdd: string[]) =>
    Array.from(new Set([...existingTags, ...tagsToAdd])).sort()

  const removeTagsUpdater = (existingTags: string[], tagsToRemove: string[]) => {
    const toRemoveSet = new Set(tagsToRemove)
    return existingTags.filter((t) => !toRemoveSet.has(t))
  }

  const replaceTagsUpdater = (_existingTags: string[], newTags: string[]) =>
    [...newTags].sort()

  return {
    toggleBatchMode: () => {
      set((state) => ({
        isBatchMode: !state.isBatchMode,
        selectedDomainKeys: new Set(),
      }))
    },

    toggleDomainSelection: (accountId: string, domainId: string) => {
      set((state) => {
        const key = `${accountId}::${domainId}`
        const next = new Set(state.selectedDomainKeys)
        if (next.has(key)) {
          next.delete(key)
        } else {
          next.add(key)
        }
        return { selectedDomainKeys: next }
      })
    },

    selectAllDomains: (accountId: string) => {
      const { domainsByAccount, selectedDomainKeys } = get()
      const cache = domainsByAccount[accountId]
      if (!cache) return

      const keys = cache.domains.map((d) => `${accountId}::${d.id}`)
      set({
        selectedDomainKeys: new Set([...selectedDomainKeys, ...keys]),
      })
    },

    clearDomainSelection: () => {
      set({ selectedDomainKeys: new Set() })
    },

    batchAddTags: async (tags: string[]) => {
      await executeBatchTagOperation(
        get().selectedDomainKeys,
        tags,
        domainMetadataService.batchAddTags.bind(domainMetadataService),
        mergeTagsUpdater,
        "domain.tags.batchAddSuccess",
        "domain.tags.batchAddPartial",
        set,
        get
      )
    },

    batchRemoveTags: async (tags: string[]) => {
      await executeBatchTagOperation(
        get().selectedDomainKeys,
        tags,
        domainMetadataService.batchRemoveTags.bind(domainMetadataService),
        removeTagsUpdater,
        "domain.tags.batchRemoveSuccess",
        "domain.tags.batchRemovePartial",
        set,
        get
      )
    },

    batchSetTags: async (tags: string[]) => {
      await executeBatchTagOperation(
        get().selectedDomainKeys,
        tags,
        domainMetadataService.batchSetTags.bind(domainMetadataService),
        replaceTagsUpdater,
        "domain.tags.batchSetSuccess",
        "domain.tags.batchSetPartial",
        set,
        get
      )
    },
  }
}
