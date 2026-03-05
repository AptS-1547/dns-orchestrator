import { domainMetadataService } from "@/services"
import { logger } from "@/lib/logger"
import { useAccountStore } from "../accountStore"
import {
  type DomainGet,
  type DomainSet,
  buildMetadataWithTags,
  updateDomainInCache,
} from "../domainStore.types"

export function createDomainMetadataSlice(set: DomainSet, get: DomainGet) {
  return {
    toggleFavorite: async (accountId: string, domainId: string) => {
      const response = await domainMetadataService.toggleFavorite(accountId, domainId)

      if (!response.success || response.data === undefined) {
        logger.error("Failed to toggle favorite:", response.error)
        return
      }

      const newFavoriteState = response.data

      set((state) => {
        const cache = state.domainsByAccount[accountId]
        if (!cache) return {}

        const domains = cache.domains.map((d) => {
          if (d.id === domainId) {
            const existingFavoritedAt = d.metadata?.favoritedAt

            return {
              ...d,
              metadata: {
                isFavorite: newFavoriteState,
                tags: d.metadata?.tags ?? [],
                color: d.metadata?.color || "none",
                updatedAt: new Date().toISOString(),
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

      get().saveToStorage()
    },

    getFavoriteDomains: () => {
      const { domainsByAccount } = get()
      const { accounts } = useAccountStore.getState()

      const favorites: import("../domainStore.types").FavoriteDomain[] = []

      Object.entries(domainsByAccount).forEach(([accountId, cache]) => {
        const account = accounts.find((a) => a.id === accountId)
        if (!(account && cache?.domains)) return

        cache.domains.forEach((domain) => {
          if (domain.metadata?.isFavorite) {
            favorites.push({
              accountId,
              domainId: domain.id,
              domainName: domain.name,
              accountName: account.name,
              provider: domain.provider,
              favoritedAt: new Date(
                domain.metadata.favoritedAt ?? domain.metadata.updatedAt
              ).getTime(),
            })
          }
        })
      })

      return favorites.sort((a, b) => b.favoritedAt - a.favoritedAt)
    },

    addTag: async (accountId: string, domainId: string, tag: string) => {
      const response = await domainMetadataService.addTag(accountId, domainId, tag)

      if (!(response.success && response.data)) {
        logger.error("Failed to add tag:", response.error)
        return
      }

      const newTags = response.data
      set((state) =>
        updateDomainInCache(state, accountId, domainId, (d) => buildMetadataWithTags(d, newTags))
      )
      get().saveToStorage()
    },

    removeTag: async (accountId: string, domainId: string, tag: string) => {
      const response = await domainMetadataService.removeTag(accountId, domainId, tag)

      if (!(response.success && response.data)) {
        logger.error("Failed to remove tag:", response.error)
        return
      }

      const newTags = response.data
      set((state) =>
        updateDomainInCache(state, accountId, domainId, (d) => buildMetadataWithTags(d, newTags))
      )
      get().saveToStorage()
    },

    setTags: async (accountId: string, domainId: string, tags: string[]) => {
      const response = await domainMetadataService.setTags(accountId, domainId, tags)

      if (!(response.success && response.data)) {
        logger.error("Failed to set tags:", response.error)
        return
      }

      const newTags = response.data
      set((state) =>
        updateDomainInCache(state, accountId, domainId, (d) => buildMetadataWithTags(d, newTags))
      )
      get().saveToStorage()

      // 清理筛选器中不存在的标签
      const allUsedTags = get().getAllUsedTags()
      const { selectedTags } = get()
      if (selectedTags.size > 0) {
        const validTags = Array.from(selectedTags).filter((tag) => allUsedTags.includes(tag))
        if (validTags.length !== selectedTags.size) {
          set({ selectedTags: new Set(validTags) })
        }
      }
    },

    updateMetadata: async (
      accountId: string,
      domainId: string,
      update: import("@/types").DomainMetadataUpdate
    ) => {
      const response = await domainMetadataService.updateMetadata(accountId, domainId, update)

      if (!(response.success && response.data)) {
        logger.error("Failed to update metadata:", response.error)
        return
      }

      const newMetadata = response.data
      set((state) => updateDomainInCache(state, accountId, domainId, () => newMetadata))
      get().saveToStorage()
    },

    setColor: async (accountId: string, domainId: string, color: string | null) => {
      await get().updateMetadata(accountId, domainId, { color: color || "none" })
    },

    setNote: async (accountId: string, domainId: string, note: string | null) => {
      await get().updateMetadata(accountId, domainId, { note: note === null ? null : note })
    },
  }
}
