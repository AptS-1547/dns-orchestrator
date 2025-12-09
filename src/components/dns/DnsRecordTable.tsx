import { ArrowDown, ArrowUp, ArrowUpDown, Loader2 } from "lucide-react"
import { useCallback, useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useDebouncedCallback } from "use-debounce"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { TIMING } from "@/constants"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { useDnsStore } from "@/stores"
import type { DnsRecord } from "@/types"
import { DnsBatchActionBar } from "./DnsBatchActionBar"
import { DnsRecordCard } from "./DnsRecordCard"
import { DnsRecordForm } from "./DnsRecordForm"
import { DnsRecordRow } from "./DnsRecordRow"
import { DnsTableToolbar } from "./DnsTableToolbar"
import { type SortField, useDnsTableSort } from "./useDnsTableSort"

interface DnsRecordTableProps {
  accountId: string
  domainId: string
  supportsProxy: boolean
}

export function DnsRecordTable({ accountId, domainId, supportsProxy }: DnsRecordTableProps) {
  const { t } = useTranslation()
  const isMobile = useIsMobile()
  const {
    records,
    isLoading,
    isLoadingMore,
    isDeleting,
    hasMore,
    totalCount,
    currentDomainId,
    keyword,
    recordType,
    setKeyword,
    setRecordType,
    fetchRecords,
    fetchMoreRecords,
    deleteRecord,
    selectedRecordIds,
    isSelectMode,
    isBatchDeleting,
    toggleSelectMode,
    toggleRecordSelection,
    selectAllRecords,
    clearSelection,
    batchDeleteRecords,
  } = useDnsStore()

  const [showAddForm, setShowAddForm] = useState(false)
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null)
  const [deletingRecord, setDeletingRecord] = useState<DnsRecord | null>(null)
  const [showBatchDeleteConfirm, setShowBatchDeleteConfirm] = useState(false)
  const sentinelRef = useRef<HTMLElement | null>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  // 使用排序 hook
  const { sortField, sortDirection, sortedRecords, handleSort } = useDnsTableSort(records)

  // 统一的 ref callback
  const setSentinelRef = useCallback((node: HTMLElement | null) => {
    sentinelRef.current = node
  }, [])

  // 防抖搜索
  const debouncedSearch = useDebouncedCallback((searchKeyword: string) => {
    fetchRecords(accountId, domainId, searchKeyword, recordType)
  }, TIMING.DEBOUNCE_DELAY)

  // 处理搜索输入变化
  const handleSearchChange = (value: string) => {
    setKeyword(value)
    debouncedSearch(value)
  }

  // 处理类型选择变化
  const handleTypeChange = (type: string) => {
    const newType = recordType === type ? "" : type
    setRecordType(newType)
    fetchRecords(accountId, domainId, keyword, newType)
  }

  // 清除所有筛选
  const clearFilters = () => {
    setKeyword("")
    setRecordType("")
    fetchRecords(accountId, domainId, "", "")
  }

  useEffect(() => {
    fetchRecords(accountId, domainId)
  }, [accountId, domainId, fetchRecords])

  // 无限滚动 IntersectionObserver
  const handleObserver = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries
      if (entry.isIntersecting && hasMore && !isLoadingMore) {
        fetchMoreRecords(accountId, domainId)
      }
    },
    [hasMore, isLoadingMore, fetchMoreRecords, accountId, domainId]
  )

  useEffect(() => {
    const sentinel = sentinelRef.current
    const scrollContainer = scrollContainerRef.current
    if (!(sentinel && scrollContainer)) return

    const observer = new IntersectionObserver(handleObserver, {
      root: scrollContainer,
      rootMargin: "100px",
    })
    observer.observe(sentinel)

    return () => observer.disconnect()
  }, [handleObserver])

  const hasActiveFilters = !!(keyword || recordType)

  // 排序图标组件
  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) {
      return <ArrowUpDown className="ml-1 h-3 w-3 opacity-40" />
    }
    if (sortDirection === "asc") {
      return <ArrowUp className="ml-1 h-3 w-3" />
    }
    return <ArrowDown className="ml-1 h-3 w-3" />
  }

  const handleDelete = (record: DnsRecord) => setDeletingRecord(record)
  const handleEdit = (record: DnsRecord) => {
    setEditingRecord(record)
    setShowAddForm(true)
  }
  const handleFormClose = () => {
    setShowAddForm(false)
    setEditingRecord(null)
  }

  const confirmDelete = async () => {
    if (!deletingRecord) return
    await deleteRecord(accountId, deletingRecord.id, domainId)
    setDeletingRecord(null)
  }

  // 只有域名切换时才显示全屏 loading
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
      {/* Toolbar */}
      <DnsTableToolbar
        totalCount={totalCount}
        isLoading={isLoading}
        keyword={keyword}
        recordType={recordType}
        hasRecords={records.length > 0}
        isSelectMode={isSelectMode}
        onSearchChange={handleSearchChange}
        onTypeChange={handleTypeChange}
        onClearFilters={clearFilters}
        onRefresh={() => fetchRecords(accountId, domainId, keyword, recordType)}
        onToggleSelectMode={toggleSelectMode}
        onAdd={() => setShowAddForm(true)}
      />

      {/* Table / Card List */}
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
            sortField={sortField}
            sortDirection={sortDirection}
            onSort={handleSort}
            onEdit={handleEdit}
            onDelete={handleDelete}
            onToggleSelect={toggleRecordSelection}
            onSelectAll={selectAllRecords}
            onClearSelection={clearSelection}
            setSentinelRef={setSentinelRef}
            SortIcon={SortIcon}
          />
        )}
      </div>

      {/* Add/Edit Form Dialog */}
      {showAddForm && (
        <DnsRecordForm
          accountId={accountId}
          domainId={domainId}
          record={editingRecord}
          onClose={handleFormClose}
          supportsProxy={supportsProxy}
        />
      )}

      {/* Delete Confirmation Dialog */}
      <AlertDialog
        open={!!deletingRecord}
        onOpenChange={(open) => !open && setDeletingRecord(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("dns.deleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("dns.deleteConfirmDesc", { name: deletingRecord?.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Batch Delete Confirmation Dialog */}
      <AlertDialog open={showBatchDeleteConfirm} onOpenChange={setShowBatchDeleteConfirm}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t("dns.batchDeleteConfirm")}</AlertDialogTitle>
            <AlertDialogDescription>
              {t("dns.batchDeleteConfirmDesc", { count: selectedRecordIds.size })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
            <AlertDialogAction
              onClick={async () => {
                setShowBatchDeleteConfirm(false)
                await batchDeleteRecords(accountId, domainId)
              }}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              {t("common.delete")}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* Batch Action Bar */}
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

// ============ 移动端卡片列表 ============

interface MobileCardListProps {
  records: DnsRecord[]
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  isSelectMode: boolean
  selectedRecordIds: Set<string>
  hasActiveFilters: boolean
  supportsProxy: boolean
  onEdit: (record: DnsRecord) => void
  onDelete: (record: DnsRecord) => void
  onToggleSelect: (id: string) => void
  onSelectAll: () => void
  onClearSelection: () => void
  setSentinelRef: (node: HTMLElement | null) => void
}

function MobileCardList({
  records,
  isLoading,
  isLoadingMore,
  isDeleting,
  isSelectMode,
  selectedRecordIds,
  hasActiveFilters,
  supportsProxy,
  onEdit,
  onDelete,
  onToggleSelect,
  onSelectAll,
  onClearSelection,
  setSentinelRef,
}: MobileCardListProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-col gap-3 p-4">
      {/* 选择模式下显示全选行 */}
      {isSelectMode && records.length > 0 && (
        <div className="flex items-center gap-2 rounded-lg border bg-muted/50 p-3">
          <Checkbox
            checked={records.every((r) => selectedRecordIds.has(r.id))}
            onCheckedChange={(checked) => {
              if (checked) onSelectAll()
              else onClearSelection()
            }}
          />
          <span className="text-muted-foreground text-sm">{t("common.selectAll")}</span>
        </div>
      )}

      {records.length === 0 ? (
        isLoading ? (
          <div className="py-8 text-center">
            <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="py-8 text-center text-muted-foreground">
            {hasActiveFilters ? t("common.noMatch") : t("dns.noRecords")}
          </div>
        )
      ) : (
        <>
          {records.map((record) => (
            <DnsRecordCard
              key={record.id}
              record={record}
              onEdit={() => onEdit(record)}
              onDelete={() => onDelete(record)}
              disabled={isDeleting}
              showProxy={supportsProxy}
              isSelectMode={isSelectMode}
              isSelected={selectedRecordIds.has(record.id)}
              onToggleSelect={() => onToggleSelect(record.id)}
            />
          ))}
          <div ref={setSentinelRef} className="h-1" />
          {isLoadingMore && (
            <div className="py-4 text-center">
              <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          )}
        </>
      )}
    </div>
  )
}

// ============ 桌面端表格 ============

interface DesktopTableProps {
  records: DnsRecord[]
  isLoading: boolean
  isLoadingMore: boolean
  isDeleting: boolean
  isSelectMode: boolean
  selectedRecordIds: Set<string>
  hasActiveFilters: boolean
  supportsProxy: boolean
  sortField: SortField | null
  sortDirection: "asc" | "desc" | null
  onSort: (field: SortField) => void
  onEdit: (record: DnsRecord) => void
  onDelete: (record: DnsRecord) => void
  onToggleSelect: (id: string) => void
  onSelectAll: () => void
  onClearSelection: () => void
  setSentinelRef: (node: HTMLElement | null) => void
  SortIcon: React.ComponentType<{ field: SortField }>
}

function DesktopTable({
  records,
  isLoading,
  isLoadingMore,
  isDeleting,
  isSelectMode,
  selectedRecordIds,
  hasActiveFilters,
  supportsProxy,
  onSort,
  onEdit,
  onDelete,
  onToggleSelect,
  onSelectAll,
  onClearSelection,
  setSentinelRef,
  SortIcon,
}: DesktopTableProps) {
  const { t } = useTranslation()
  const colSpan = (supportsProxy ? 6 : 5) + (isSelectMode ? 1 : 0)

  return (
    <Table>
      <TableHeader className="sticky top-0 z-10 bg-background">
        <TableRow>
          {isSelectMode && (
            <TableHead className="w-10">
              <Checkbox
                checked={records.length > 0 && records.every((r) => selectedRecordIds.has(r.id))}
                onCheckedChange={(checked) => {
                  if (checked) onSelectAll()
                  else onClearSelection()
                }}
              />
            </TableHead>
          )}
          <TableHead
            className="w-16 cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("type")}
          >
            <div className="flex items-center">
              {t("common.type")}
              <SortIcon field="type" />
            </div>
          </TableHead>
          <TableHead
            className="w-28 cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("name")}
          >
            <div className="flex items-center">
              {t("dns.name")}
              <SortIcon field="name" />
            </div>
          </TableHead>
          <TableHead
            className="cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("value")}
          >
            <div className="flex items-center">
              {t("dns.value")}
              <SortIcon field="value" />
            </div>
          </TableHead>
          <TableHead
            className="w-20 cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("ttl")}
          >
            <div className="flex items-center">
              {t("dns.ttl")}
              <SortIcon field="ttl" />
            </div>
          </TableHead>
          {supportsProxy && <TableHead className="w-12">{t("dns.proxy")}</TableHead>}
          <TableHead className="w-16 text-right">{t("dns.actions")}</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {records.length === 0 ? (
          <TableRow>
            <TableCell colSpan={colSpan} className="py-8 text-center text-muted-foreground">
              {isLoading ? (
                <Loader2 className="mx-auto h-5 w-5 animate-spin" />
              ) : hasActiveFilters ? (
                t("common.noMatch")
              ) : (
                t("dns.noRecords")
              )}
            </TableCell>
          </TableRow>
        ) : (
          <>
            {records.map((record) => (
              <TableRow key={record.id}>
                {isSelectMode && (
                  <TableCell className="w-10">
                    <Checkbox
                      checked={selectedRecordIds.has(record.id)}
                      onCheckedChange={() => onToggleSelect(record.id)}
                    />
                  </TableCell>
                )}
                <DnsRecordRow
                  record={record}
                  onEdit={() => onEdit(record)}
                  onDelete={() => onDelete(record)}
                  disabled={isDeleting || isSelectMode}
                  showProxy={supportsProxy}
                  asFragment
                />
              </TableRow>
            ))}
            <TableRow ref={setSentinelRef} className="h-1 border-0">
              <TableCell colSpan={colSpan} className="p-0" />
            </TableRow>
            {isLoadingMore && (
              <TableRow className="border-0">
                <TableCell colSpan={colSpan} className="py-4 text-center">
                  <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
                </TableCell>
              </TableRow>
            )}
          </>
        )}
      </TableBody>
    </Table>
  )
}
