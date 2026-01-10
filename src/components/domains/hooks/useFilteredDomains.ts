import { useMemo } from "react"
import { useDomainStore } from "@/stores"
import type { Domain } from "@/types"

interface UseFilteredDomainsOptions {
  accountId: string
  searchQuery: string
}

export function useFilteredDomains({
  accountId,
  searchQuery,
}: UseFilteredDomainsOptions): Domain[] {
  const domains = useDomainStore((state) => state.domainsByAccount[accountId]?.domains ?? [])
  const selectedTags = useDomainStore((state) => state.selectedTags)

  return useMemo(() => {
    let filtered = domains

    // Search filter
    if (searchQuery.trim()) {
      const query = searchQuery.toLowerCase()
      filtered = filtered.filter((d) => d.name.toLowerCase().includes(query))
    }

    // Tag filter (any match)
    if (selectedTags.size > 0) {
      filtered = filtered.filter((domain) => {
        const tags = domain.metadata?.tags ?? []
        return Array.from(selectedTags).some((tag) => tags.includes(tag))
      })
    }

    return filtered
  }, [domains, searchQuery, selectedTags])
}
