import {
  CheckSquare,
  ChevronDown,
  Filter,
  Plus,
  RefreshCw,
  Search,
  Settings,
  Wand2,
  X,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
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
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { useIsMobile } from "@/hooks/useMediaQuery"
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
  /** 添加记录（高级模式） */
  onAdd: () => void
  /** 添加记录（向导模式） */
  onAddWizard: () => void
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
  onAddWizard,
}: DnsTableToolbarProps) {
  const { t } = useTranslation()
  const isMobile = useIsMobile()
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

          {/* 分页大小选择器（仅传统分页模式 + 桌面端显示） */}
          {paginationMode === "paginated" && (
            <div className="ml-4 hidden items-center gap-2 md:flex">
              <span className="text-muted-foreground text-sm">{t("common.perPage")}</span>
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
              <span className="text-muted-foreground text-sm">{t("common.items")}</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          {/* Select 按钮：移动端纯图标 */}
          {isMobile ? (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant={isSelectMode ? "secondary" : "outline"}
                    size="icon"
                    className="h-9 w-9"
                    onClick={onToggleSelectMode}
                    disabled={!hasRecords}
                  >
                    {isSelectMode ? <X className="h-4 w-4" /> : <CheckSquare className="h-4 w-4" />}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {isSelectMode ? t("common.cancel") : t("dns.batchSelect")}
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          ) : (
            <Button
              variant={isSelectMode ? "secondary" : "outline"}
              size="sm"
              onClick={onToggleSelectMode}
              disabled={!hasRecords}
            >
              <CheckSquare className="mr-2 h-4 w-4" />
              {isSelectMode ? t("common.cancel") : t("dns.batchSelect")}
            </Button>
          )}

          {/* Add Record 按钮：移动端纯图标 + 下拉 */}
          {!isSelectMode &&
            (isMobile ? (
              <DropdownMenu>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <DropdownMenuTrigger asChild>
                        <Button size="icon" className="h-9 w-9">
                          <Plus className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                    </TooltipTrigger>
                    <TooltipContent>{t("dns.addRecord")}</TooltipContent>
                  </Tooltip>
                </TooltipProvider>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={onAdd}>
                    <Settings className="mr-2 h-4 w-4" />
                    {t("dns.wizard.advancedMode")}
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={onAddWizard}>
                    <Wand2 className="mr-2 h-4 w-4" />
                    {t("dns.wizard.wizardMode")}
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : (
              <div className="flex items-center">
                <Button size="sm" onClick={onAdd} className="rounded-r-none">
                  <Plus className="mr-2 h-4 w-4" />
                  {t("dns.addRecord")}
                </Button>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      size="sm"
                      className="rounded-l-none border-l border-l-primary-foreground/20 px-2"
                    >
                      <ChevronDown className="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem onClick={onAdd}>
                      <Settings className="mr-2 h-4 w-4" />
                      {t("dns.wizard.advancedMode")}
                    </DropdownMenuItem>
                    <DropdownMenuItem onClick={onAddWizard}>
                      <Wand2 className="mr-2 h-4 w-4" />
                      {t("dns.wizard.wizardMode")}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            ))}
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
            <Button
              variant="outline"
              size={isMobile ? "icon" : "sm"}
              className={cn("h-8", isMobile && "w-8")}
            >
              <Filter className={cn("h-4 w-4", !isMobile && "mr-2")} />
              {!isMobile && (recordType || t("common.type"))}
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

        {hasActiveFilters &&
          (isMobile ? (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onClearFilters}>
                    <X className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>{t("common.clearFilter")}</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          ) : (
            <Button variant="ghost" size="sm" className="h-8" onClick={onClearFilters}>
              <X className="mr-1 h-4 w-4" />
              {t("common.clearFilter")}
            </Button>
          ))}
      </div>
    </div>
  )
}
