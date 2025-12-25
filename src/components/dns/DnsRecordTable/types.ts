import type { DnsRecord } from "@/types"
import type { SortField } from "../useDnsTableSort"

export interface DnsRecordTableProps {
  accountId: string
  domainId: string
  supportsProxy: boolean
}

export interface MobileCardListProps {
  records: DnsRecord[]
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  isSelectMode: boolean
  selectedRecordIds: Set<string>
  hasActiveFilters: boolean
  supportsProxy: boolean
  domainName?: string
  onEdit: (record: DnsRecord) => void
  onDelete: (record: DnsRecord) => void
  onToggleSelect: (id: string) => void
  onSelectAll: () => void
  onClearSelection: () => void
  setSentinelRef: (node: HTMLElement | null) => void
}

export interface DesktopTableProps {
  records: DnsRecord[]
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  isSelectMode: boolean
  selectedRecordIds: Set<string>
  hasActiveFilters: boolean
  supportsProxy: boolean
  domainName?: string
  sortField: SortField | null
  sortDirection: "asc" | "desc" | null
  onSort: (field: SortField) => void
  onEdit: (record: DnsRecord) => void
  onDelete: (record: DnsRecord) => void
  onToggleSelect: (id: string) => void
  onSelectAll: () => void
  onClearSelection: () => void
  setSentinelRef: (node: HTMLElement | null) => void
}
