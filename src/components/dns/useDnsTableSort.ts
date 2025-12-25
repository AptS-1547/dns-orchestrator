import { useMemo, useState } from "react"
import type { DnsRecord } from "@/types"

export type SortField = "type" | "name" | "value" | "ttl"
export type SortDirection = "asc" | "desc" | null

export interface UseDnsTableSortResult {
  sortField: SortField | null
  sortDirection: SortDirection
  sortedRecords: DnsRecord[]
  handleSort: (field: SortField) => void
}

/** 获取记录的显示值（用于排序） */
function getRecordDisplayValue(record: DnsRecord): string {
  const { data } = record

  switch (data.type) {
    case "A":
    case "AAAA":
      return data.content.address
    case "CNAME":
      return data.content.target
    case "MX":
      return data.content.exchange
    case "TXT":
      return data.content.text
    case "NS":
      return data.content.nameserver
    case "SRV":
      return data.content.target
    case "CAA":
      return data.content.value
  }
}

/**
 * DNS 记录表格排序 hook
 */
export function useDnsTableSort(records: DnsRecord[]): UseDnsTableSortResult {
  const [sortField, setSortField] = useState<SortField | null>(null)
  const [sortDirection, setSortDirection] = useState<SortDirection>(null)

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

  // 排序后的记录
  const sortedRecords = useMemo(() => {
    if (!(sortField && sortDirection)) return records

    return [...records].sort((a, b) => {
      let aVal: string | number
      let bVal: string | number

      switch (sortField) {
        case "type":
          aVal = a.data.type
          bVal = b.data.type
          break
        case "name":
          aVal = a.name
          bVal = b.name
          break
        case "value":
          aVal = getRecordDisplayValue(a)
          bVal = getRecordDisplayValue(b)
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

  return {
    sortField,
    sortDirection,
    sortedRecords,
    handleSort,
  }
}
