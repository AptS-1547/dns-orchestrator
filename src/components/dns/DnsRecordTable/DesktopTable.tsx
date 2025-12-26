import { ArrowDown, ArrowUp, ArrowUpDown, Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { DnsRecordRow } from "../DnsRecordRow"
import type { SortField } from "../useDnsTableSort"
import type { DesktopTableProps } from "./types"

// 排序图标 - 提取到组件外部避免每次渲染重建
function SortIcon({
  field,
  sortField,
  sortDirection,
}: {
  field: SortField
  sortField: SortField | null
  sortDirection: "asc" | "desc" | null
}) {
  if (sortField !== field) {
    return <ArrowUpDown className="ml-1 h-3 w-3 opacity-40" />
  }
  if (sortDirection === "asc") {
    return <ArrowUp className="ml-1 h-3 w-3" />
  }
  return <ArrowDown className="ml-1 h-3 w-3" />
}

export function DesktopTable({
  records,
  isLoading,
  isLoadingMore,
  isDeleting,
  isSelectMode,
  selectedRecordIds,
  hasActiveFilters,
  supportsProxy,
  domainName,
  sortField,
  sortDirection,
  onSort,
  onEdit,
  onDelete,
  onToggleSelect,
  onSelectAll,
  onClearSelection,
  setSentinelRef,
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
              <SortIcon field="type" sortField={sortField} sortDirection={sortDirection} />
            </div>
          </TableHead>
          <TableHead
            className="w-28 cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("name")}
          >
            <div className="flex items-center">
              {t("dns.name")}
              <SortIcon field="name" sortField={sortField} sortDirection={sortDirection} />
            </div>
          </TableHead>
          <TableHead
            className="cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("value")}
          >
            <div className="flex items-center">
              {t("dns.value")}
              <SortIcon field="value" sortField={sortField} sortDirection={sortDirection} />
            </div>
          </TableHead>
          <TableHead
            className="w-20 cursor-pointer select-none hover:bg-muted/50"
            onClick={() => onSort("ttl")}
          >
            <div className="flex items-center">
              {t("dns.ttl")}
              <SortIcon field="ttl" sortField={sortField} sortDirection={sortDirection} />
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
                  domainName={domainName}
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
