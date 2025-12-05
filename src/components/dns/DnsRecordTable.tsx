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
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Input } from "@/components/ui/input"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { cn } from "@/lib/utils"
import { useDnsStore } from "@/stores"
import type { DnsRecord } from "@/types"
import {
  ArrowDown,
  ArrowUp,
  ArrowUpDown,
  Filter,
  Loader2,
  Plus,
  RefreshCw,
  Search,
  X,
} from "lucide-react"
import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useDebouncedCallback } from "use-debounce"
import { DnsRecordForm } from "./DnsRecordForm"
import { DnsRecordRow } from "./DnsRecordRow"

type SortField = "type" | "name" | "value" | "ttl"
type SortDirection = "asc" | "desc" | null

interface DnsRecordTableProps {
  accountId: string
  domainId: string
  supportsProxy: boolean
}

// 可用的记录类型列表
const RECORD_TYPES = ["A", "AAAA", "CNAME", "MX", "TXT", "NS", "SRV", "CAA"]

export function DnsRecordTable({ accountId, domainId, supportsProxy }: DnsRecordTableProps) {
  const { t } = useTranslation()
  const {
    records,
    isLoading,
    isLoadingMore,
    isDeleting,
    hasMore,
    totalCount,
    currentDomainId,
    keyword: storeKeyword,
    recordType: storeRecordType,
    fetchRecords,
    fetchMoreRecords,
    deleteRecord,
  } = useDnsStore()
  const [showAddForm, setShowAddForm] = useState(false)
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null)
  const [deletingRecord, setDeletingRecord] = useState<DnsRecord | null>(null)
  const [sortField, setSortField] = useState<SortField | null>(null)
  const [sortDirection, setSortDirection] = useState<SortDirection>(null)
  // 本地搜索输入状态（用于即时显示）
  const [searchInput, setSearchInput] = useState("")
  // 选中的类型（本地状态）
  const [selectedType, setSelectedType] = useState("")
  const sentinelRef = useRef<HTMLTableRowElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  // 防抖搜索
  const debouncedSearch = useDebouncedCallback((keyword: string) => {
    fetchRecords(accountId, domainId, keyword, selectedType)
  }, 300)

  // 处理搜索输入变化
  const handleSearchChange = (value: string) => {
    setSearchInput(value)
    debouncedSearch(value)
  }

  // 处理类型选择变化
  const handleTypeChange = (type: string) => {
    const newType = selectedType === type ? "" : type
    setSelectedType(newType)
    fetchRecords(accountId, domainId, searchInput, newType)
  }

  // 清除所有筛选
  const clearFilters = () => {
    setSearchInput("")
    setSelectedType("")
    fetchRecords(accountId, domainId, "", "")
  }

  useEffect(() => {
    // 初始加载时重置本地状态
    setSearchInput(storeKeyword)
    setSelectedType(storeRecordType)
    fetchRecords(accountId, domainId)
  }, [accountId, domainId]) // 只在账户/域名变化时重新加载

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

  // 处理排序点击
  const handleSort = (field: SortField) => {
    if (sortField === field) {
      // 同一列：asc -> desc -> null 循环
      if (sortDirection === "asc") {
        setSortDirection("desc")
      } else if (sortDirection === "desc") {
        setSortDirection(null)
        setSortField(null)
      } else {
        setSortDirection("asc")
      }
    } else {
      // 新列：从 asc 开始
      setSortField(field)
      setSortDirection("asc")
    }
  }

  const hasActiveFilters = searchInput || selectedType

  // 排序后的记录（搜索过滤已由后端完成）
  const sortedRecords = useMemo(() => {
    if (!(sortField && sortDirection)) return records

    return [...records].sort((a, b) => {
      let aVal: string | number
      let bVal: string | number

      switch (sortField) {
        case "type":
          aVal = a.type
          bVal = b.type
          break
        case "name":
          aVal = a.name
          bVal = b.name
          break
        case "value":
          aVal = a.value
          bVal = b.value
          break
        case "ttl":
          aVal = a.ttl
          bVal = b.ttl
          break
        default:
          return 0
      }

      if (typeof aVal === "number" && typeof bVal === "number") {
        return sortDirection === "asc" ? aVal - bVal : bVal - aVal
      }

      const comparison = String(aVal).localeCompare(String(bVal))
      return sortDirection === "asc" ? comparison : -comparison
    })
  }, [records, sortField, sortDirection])

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

  const handleDelete = (record: DnsRecord) => {
    setDeletingRecord(record)
  }

  const confirmDelete = async () => {
    if (!deletingRecord) return
    await deleteRecord(accountId, deletingRecord.id, domainId)
    setDeletingRecord(null)
  }

  const handleEdit = (record: DnsRecord) => {
    setEditingRecord(record)
    setShowAddForm(true)
  }

  const handleFormClose = () => {
    setShowAddForm(false)
    setEditingRecord(null)
  }

  // 只有初次加载（domain 切换）才显示全屏 loading
  // 搜索时即使结果为空也不显示全屏 loading
  const isInitialLoading = isLoading && records.length === 0 && currentDomainId !== domainId

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
      <div className="flex flex-col gap-3 border-b bg-muted/30 px-6 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8"
              onClick={() => fetchRecords(accountId, domainId, searchInput, selectedType)}
              disabled={isLoading}
            >
              <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
            </Button>
            <span className="text-muted-foreground text-sm">{t("common.total")}</span>
            <Badge variant="secondary">{totalCount}</Badge>
            <span className="text-muted-foreground text-sm">{t("common.records")}</span>
          </div>
          <Button size="sm" onClick={() => setShowAddForm(true)}>
            <Plus className="mr-2 h-4 w-4" />
            {t("dns.addRecord")}
          </Button>
        </div>

        {/* 搜索和筛选 */}
        <div className="flex items-center gap-2">
          <div className="relative max-w-sm flex-1">
            <Search className="-translate-y-1/2 absolute top-1/2 left-2.5 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder={t("dns.searchPlaceholder")}
              value={searchInput}
              onChange={(e) => handleSearchChange(e.target.value)}
              className="h-8 pr-8 pl-8"
            />
            {searchInput && (
              <Button
                variant="ghost"
                size="icon"
                className="-translate-y-1/2 absolute top-1/2 right-1 h-6 w-6"
                onClick={() => handleSearchChange("")}
              >
                <X className="h-3 w-3" />
              </Button>
            )}
          </div>

          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm" className="h-8">
                <Filter className="mr-2 h-4 w-4" />
                {selectedType || t("common.type")}
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start">
              {RECORD_TYPES.map((type) => (
                <DropdownMenuCheckboxItem
                  key={type}
                  checked={selectedType === type}
                  onCheckedChange={() => handleTypeChange(type)}
                >
                  {type}
                </DropdownMenuCheckboxItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {hasActiveFilters && (
            <Button variant="ghost" size="sm" className="h-8" onClick={clearFilters}>
              <X className="mr-1 h-4 w-4" />
              {t("common.clearFilter")}
            </Button>
          )}
        </div>
      </div>

      {/* Table */}
      <div ref={scrollContainerRef} className="min-h-0 flex-1 overflow-auto">
        <Table>
          <TableHeader className="sticky top-0 z-10 bg-background">
            <TableRow>
              <TableHead
                className="w-16 cursor-pointer select-none hover:bg-muted/50"
                onClick={() => handleSort("type")}
              >
                <div className="flex items-center">
                  {t("common.type")}
                  <SortIcon field="type" />
                </div>
              </TableHead>
              <TableHead
                className="w-28 cursor-pointer select-none hover:bg-muted/50"
                onClick={() => handleSort("name")}
              >
                <div className="flex items-center">
                  {t("dns.name")}
                  <SortIcon field="name" />
                </div>
              </TableHead>
              <TableHead
                className="cursor-pointer select-none hover:bg-muted/50"
                onClick={() => handleSort("value")}
              >
                <div className="flex items-center">
                  {t("dns.value")}
                  <SortIcon field="value" />
                </div>
              </TableHead>
              <TableHead
                className="w-20 cursor-pointer select-none hover:bg-muted/50"
                onClick={() => handleSort("ttl")}
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
            {sortedRecords.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={supportsProxy ? 6 : 5}
                  className="py-8 text-center text-muted-foreground"
                >
                  {hasActiveFilters ? t("common.noMatch") : t("dns.noRecords")}
                </TableCell>
              </TableRow>
            ) : (
              <>
                {sortedRecords.map((record) => (
                  <DnsRecordRow
                    key={record.id}
                    record={record}
                    onEdit={() => handleEdit(record)}
                    onDelete={() => handleDelete(record)}
                    disabled={isDeleting}
                    showProxy={supportsProxy}
                  />
                ))}
                {/* 无限滚动触发行 */}
                <TableRow ref={sentinelRef} className="h-1 border-0">
                  <TableCell colSpan={supportsProxy ? 6 : 5} className="p-0" />
                </TableRow>
                {isLoadingMore && (
                  <TableRow className="border-0">
                    <TableCell colSpan={supportsProxy ? 6 : 5} className="py-4 text-center">
                      <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
                    </TableCell>
                  </TableRow>
                )}
              </>
            )}
          </TableBody>
        </Table>
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
    </div>
  )
}
