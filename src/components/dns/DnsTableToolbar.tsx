import { CheckSquare, Filter, Plus, RefreshCw, Search, X } from "lucide-react"
import { useTranslation } from "react-i18next"
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"
import { useDnsStore, useSettingsStore } from "@/stores"
import { RECORD_TYPES } from "@/types/dns"

interface DnsTableToolbarProps {
  /** 账户 ID */
  accountId: string
  /** 域名 ID */
  domainId: string
  /** 总记录数 */
  totalCount: number
  /** 是否正在加载 */
  isLoading: boolean
  /** 搜索关键词 */
  keyword: string
  /** 当前筛选的记录类型 */
  recordType: string
  /** 记录列表是否为空 */
  hasRecords: boolean
  /** 是否处于选择模式 */
  isSelectMode: boolean
  /** 搜索变化回调 */
  onSearchChange: (value: string) => void
  /** 类型筛选变化回调 */
  onTypeChange: (type: string) => void
  /** 清除所有筛选 */
  onClearFilters: () => void
  /** 刷新回调 */
  onRefresh: () => void
  /** 切换选择模式 */
  onToggleSelectMode: () => void
  /** 添加记录 */
  onAdd: () => void
}

export function DnsTableToolbar({
  accountId,
  domainId,
  totalCount,
  isLoading,
  keyword,
  recordType,
  hasRecords,
  isSelectMode,
  onSearchChange,
  onTypeChange,
  onClearFilters,
  onRefresh,
  onToggleSelectMode,
  onAdd,
}: DnsTableToolbarProps) {
  const { t } = useTranslation()
  const hasActiveFilters = keyword || recordType
  const paginationMode = useSettingsStore((state) => state.paginationMode)
  const pageSize = useDnsStore((state) => state.pageSize)
  const setPageSize = useDnsStore((state) => state.setPageSize)

  return (
    <div className="flex flex-col gap-3 border-b bg-muted/30 px-6 py-3">
      {/* 顶部行：刷新、统计、操作按钮 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            onClick={onRefresh}
            disabled={isLoading}
          >
            <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
          </Button>
          <span className="text-muted-foreground text-sm">{t("common.total")}</span>
          <Badge variant="secondary">{totalCount}</Badge>
          <span className="text-muted-foreground text-sm">{t("common.records")}</span>

          {/* 分页大小选择器（仅传统分页模式显示） */}
          {paginationMode === "paginated" && (
            <div className="ml-4 flex items-center gap-2">
              <span className="text-muted-foreground text-sm">每页</span>
              <Select
                value={String(pageSize)}
                onValueChange={(val) => setPageSize(accountId, domainId, Number(val))}
              >
                <SelectTrigger className="h-8 w-20">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="10">10</SelectItem>
                  <SelectItem value="20">20</SelectItem>
                  <SelectItem value="50">50</SelectItem>
                  <SelectItem value="100">100</SelectItem>
                </SelectContent>
              </Select>
              <span className="text-muted-foreground text-sm">条</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant={isSelectMode ? "secondary" : "outline"}
            size="sm"
            onClick={onToggleSelectMode}
            disabled={!hasRecords}
          >
            <CheckSquare className="mr-2 h-4 w-4" />
            {isSelectMode ? t("common.cancel") : t("dns.batchSelect")}
          </Button>
          {!isSelectMode && (
            <Button size="sm" onClick={onAdd}>
              <Plus className="mr-2 h-4 w-4" />
              {t("dns.addRecord")}
            </Button>
          )}
        </div>
      </div>

      {/* 搜索和筛选 */}
      <div className="flex items-center gap-2">
        <div className="relative max-w-sm flex-1">
          <Search className="absolute top-1/2 left-2.5 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder={t("dns.searchPlaceholder")}
            value={keyword}
            onChange={(e) => onSearchChange(e.target.value)}
            className="h-8 pr-8 pl-8"
          />
          {keyword && (
            <Button
              variant="ghost"
              size="icon"
              className="absolute top-1/2 right-1 h-6 w-6 -translate-y-1/2"
              onClick={() => onSearchChange("")}
            >
              <X className="h-3 w-3" />
            </Button>
          )}
        </div>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="h-8">
              <Filter className="mr-2 h-4 w-4" />
              {recordType || t("common.type")}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {RECORD_TYPES.map((type) => (
              <DropdownMenuCheckboxItem
                key={type}
                checked={recordType === type}
                onCheckedChange={() => onTypeChange(type)}
              >
                {type}
              </DropdownMenuCheckboxItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {hasActiveFilters && (
          <Button variant="ghost" size="sm" className="h-8" onClick={onClearFilters}>
            <X className="mr-1 h-4 w-4" />
            {t("common.clearFilter")}
          </Button>
        )}
      </div>
    </div>
  )
}
