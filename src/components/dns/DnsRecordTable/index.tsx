import { Loader2 } from "lucide-react"
import { useCallback, useEffect, useMemo, useState } from "react"
import { useDebouncedCallback } from "use-debounce"
import { useShallow } from "zustand/react/shallow"
import { TIMING } from "@/constants"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { useDnsStore, useDomainStore, useSettingsStore } from "@/stores"
import type { DnsRecord } from "@/types"
import { DnsBatchActionBar } from "../DnsBatchActionBar"
import { DnsRecordForm } from "../DnsRecordForm"
import { DnsRecordWizard } from "../DnsRecordWizard"
import { DnsTableToolbar } from "../DnsTableToolbar"
import { useDnsTableSort } from "../useDnsTableSort"
import { BatchDeleteDialog, DeleteRecordDialog } from "./DeleteDialogs"
import { DesktopTable } from "./DesktopTable"
import { DnsPagination } from "./DnsPagination"
import { MobileCardList } from "./MobileCardList"
import type { DnsRecordTableProps } from "./types"
import { useInfiniteScroll } from "./useInfiniteScroll"

export function DnsRecordTable({ accountId, domainId, supportsProxy }: DnsRecordTableProps) {
  const isMobile = useIsMobile()
  const paginationMode = useSettingsStore((state) => state.paginationMode)

  const getDomainsForAccount = useDomainStore((state) => state.getDomainsForAccount)
  const domainName = useMemo(() => {
    const domains = getDomainsForAccount(accountId)
    return domains.find((d) => d.id === domainId)?.name
  }, [getDomainsForAccount, accountId, domainId])

  const {
    records,
    isLoading,
    isLoadingMore,
    isDeleting,
    hasMore,
    totalCount,
    currentDomainId,
    page,
    pageSize,
    keyword,
    recordType,
    selectedRecordIds,
    isSelectMode,
    isBatchDeleting,
  } = useDnsStore(
    useShallow((state) => ({
      records: state.records,
      isLoading: state.isLoading,
      isLoadingMore: state.isLoadingMore,
      isDeleting: state.isDeleting,
      hasMore: state.hasMore,
      totalCount: state.totalCount,
      currentDomainId: state.currentDomainId,
      page: state.page,
      pageSize: state.pageSize,
      keyword: state.keyword,
      recordType: state.recordType,
      selectedRecordIds: state.selectedRecordIds,
      isSelectMode: state.isSelectMode,
      isBatchDeleting: state.isBatchDeleting,
    }))
  )

  const setKeyword = useDnsStore((state) => state.setKeyword)
  const setRecordType = useDnsStore((state) => state.setRecordType)
  const setPageSize = useDnsStore((state) => state.setPageSize)
  const fetchRecords = useDnsStore((state) => state.fetchRecords)
  const fetchMoreRecords = useDnsStore((state) => state.fetchMoreRecords)
  const jumpToPage = useDnsStore((state) => state.jumpToPage)
  const deleteRecord = useDnsStore((state) => state.deleteRecord)
  const toggleSelectMode = useDnsStore((state) => state.toggleSelectMode)
  const toggleRecordSelection = useDnsStore((state) => state.toggleRecordSelection)
  const selectAllRecords = useDnsStore((state) => state.selectAllRecords)
  const clearSelection = useDnsStore((state) => state.clearSelection)
  const batchDeleteRecords = useDnsStore((state) => state.batchDeleteRecords)

  const [showAddForm, setShowAddForm] = useState(false)
  const [showWizard, setShowWizard] = useState(false)
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null)
  const [deletingRecord, setDeletingRecord] = useState<DnsRecord | null>(null)
  const [showBatchDeleteConfirm, setShowBatchDeleteConfirm] = useState(false)

  const { sortField, sortDirection, sortedRecords, handleSort } = useDnsTableSort(records)

  const { scrollContainerRef, setSentinelRef } = useInfiniteScroll({
    hasMore,
    isLoadingMore,
    enabled: paginationMode === "infinite",
    onLoadMore: () => fetchMoreRecords(accountId, domainId),
  })

  const debouncedSearch = useDebouncedCallback((searchKeyword: string) => {
    fetchRecords(accountId, domainId, searchKeyword, recordType)
  }, TIMING.DEBOUNCE_DELAY)

  const handleSearchChange = useCallback(
    (value: string) => {
      setKeyword(value)
      debouncedSearch(value)
    },
    [setKeyword, debouncedSearch]
  )

  const handleTypeChange = useCallback(
    (type: string) => {
      const newType = recordType === type ? "" : type
      setRecordType(newType)
      fetchRecords(accountId, domainId, keyword, newType)
    },
    [recordType, setRecordType, fetchRecords, accountId, domainId, keyword]
  )

  const clearFilters = useCallback(() => {
    setKeyword("")
    setRecordType("")
    fetchRecords(accountId, domainId, "", "")
  }, [setKeyword, setRecordType, fetchRecords, accountId, domainId])

  useEffect(() => {
    fetchRecords(accountId, domainId)
  }, [accountId, domainId, fetchRecords])

  const hasActiveFilters = useMemo(() => !!(keyword || recordType), [keyword, recordType])

  const handleDelete = useCallback((record: DnsRecord) => setDeletingRecord(record), [])
  const handleEdit = useCallback((record: DnsRecord) => {
    setEditingRecord(record)
    setShowAddForm(true)
  }, [])
  const handleFormClose = useCallback(() => {
    setShowAddForm(false)
    setEditingRecord(null)
  }, [])

  const handleRefresh = useCallback(() => {
    fetchRecords(accountId, domainId, keyword, recordType)
  }, [fetchRecords, accountId, domainId, keyword, recordType])

  const confirmDelete = async () => {
    if (!deletingRecord) return
    await deleteRecord(accountId, deletingRecord.id, domainId)
    setDeletingRecord(null)
  }

  const isInitialLoading = isLoading && currentDomainId !== domainId

  if (isInitialLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
      </div>
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <DnsTableToolbar
        accountId={accountId}
        domainId={domainId}
        totalCount={totalCount}
        isLoading={isLoading}
        keyword={keyword}
        recordType={recordType}
        hasRecords={records.length > 0}
        isSelectMode={isSelectMode}
        onSearchChange={handleSearchChange}
        onTypeChange={handleTypeChange}
        onClearFilters={clearFilters}
        onRefresh={handleRefresh}
        onToggleSelectMode={toggleSelectMode}
        onAdd={() => setShowAddForm(true)}
        onAddWizard={() => setShowWizard(true)}
      />

      <div ref={scrollContainerRef} className="min-h-0 flex-1 overflow-auto">
        {isMobile ? (
          <MobileCardList
            records={sortedRecords}
            isLoading={isLoading}
            isLoadingMore={isLoadingMore}
            isDeleting={isDeleting}
            isSelectMode={isSelectMode}
            selectedRecordIds={selectedRecordIds}
            hasActiveFilters={hasActiveFilters}
            supportsProxy={supportsProxy}
            domainName={domainName}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleSelect={toggleRecordSelection}
            onSelectAll={selectAllRecords}
            onClearSelection={clearSelection}
            setSentinelRef={setSentinelRef}
          />
        ) : (
          <DesktopTable
            records={sortedRecords}
            isLoading={isLoading}
            isLoadingMore={isLoadingMore}
            isDeleting={isDeleting}
            isSelectMode={isSelectMode}
            selectedRecordIds={selectedRecordIds}
            hasActiveFilters={hasActiveFilters}
            supportsProxy={supportsProxy}
            domainName={domainName}
            sortField={sortField}
            sortDirection={sortDirection}
            onSort={handleSort}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleSelect={toggleRecordSelection}
            onSelectAll={selectAllRecords}
            onClearSelection={clearSelection}
            setSentinelRef={setSentinelRef}
          />
        )}
      </div>

      {paginationMode === "paginated" && totalCount > 0 && (
        <DnsPagination
          page={page}
          pageSize={pageSize}
          totalCount={totalCount}
          onPageChange={(p) => jumpToPage(accountId, domainId, p)}
          onPageSizeChange={(size) => setPageSize(accountId, domainId, size)}
        />
      )}

      {showAddForm && (
        <DnsRecordForm
          accountId={accountId}
          domainId={domainId}
          record={editingRecord}
          onClose={handleFormClose}
          supportsProxy={supportsProxy}
        />
      )}

      {showWizard && (
        <DnsRecordWizard
          accountId={accountId}
          domainId={domainId}
          onClose={() => setShowWizard(false)}
          onOpenAdvancedForm={() => setShowAddForm(true)}
        />
      )}

      <DeleteRecordDialog
        record={deletingRecord}
        isDeleting={isDeleting}
        onConfirm={confirmDelete}
        onClose={() => setDeletingRecord(null)}
      />

      <BatchDeleteDialog
        open={showBatchDeleteConfirm}
        selectedCount={selectedRecordIds.size}
        isDeleting={isBatchDeleting}
        onConfirm={async () => {
          setShowBatchDeleteConfirm(false)
          await batchDeleteRecords(accountId, domainId)
        }}
        onClose={() => setShowBatchDeleteConfirm(false)}
      />

      {isSelectMode && (
        <DnsBatchActionBar
          selectedCount={selectedRecordIds.size}
          isDeleting={isBatchDeleting}
          onClearSelection={clearSelection}
          onDelete={() => setShowBatchDeleteConfirm(true)}
        />
      )}
    </div>
  )
}
