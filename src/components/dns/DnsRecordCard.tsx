import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"
import type { DnsRecord } from "@/types"
import { MoreHorizontal, Pencil, Shield, ShieldOff, Trash2 } from "lucide-react"
import { useTranslation } from "react-i18next"

interface DnsRecordCardProps {
  record: DnsRecord
  onEdit: () => void
  onDelete: () => void
  disabled?: boolean
  showProxy?: boolean
  /** 是否处于批量选择模式 */
  isSelectMode?: boolean
  /** 是否已选中 */
  isSelected?: boolean
  /** 切换选中状态 */
  onToggleSelect?: () => void
}

const TYPE_COLORS: Record<string, string> = {
  A: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300",
  AAAA: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300",
  CNAME: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300",
  MX: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-300",
  TXT: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-300",
  NS: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-300",
  SRV: "bg-pink-100 text-pink-800 dark:bg-pink-900 dark:text-pink-300",
  CAA: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300",
}

function formatTTL(ttl: number): string {
  if (ttl === 1) return "自动"
  if (ttl < 60) return `${ttl} 秒`
  if (ttl < 3600) return `${Math.floor(ttl / 60)} 分钟`
  if (ttl < 86400) return `${Math.floor(ttl / 3600)} 小时`
  return `${Math.floor(ttl / 86400)} 天`
}

export function DnsRecordCard({
  record,
  onEdit,
  onDelete,
  disabled = false,
  showProxy = false,
  isSelectMode = false,
  isSelected = false,
  onToggleSelect,
}: DnsRecordCardProps) {
  const { t } = useTranslation()

  return (
    <Card
      className={cn("p-3", isSelectMode && "cursor-pointer", isSelected && "ring-2 ring-primary")}
      onClick={isSelectMode ? onToggleSelect : undefined}
    >
      {/* 第一行：checkbox + type + name + actions */}
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 flex-1 items-center gap-2">
          {isSelectMode && (
            <Checkbox
              checked={isSelected}
              onCheckedChange={onToggleSelect}
              onClick={(e) => e.stopPropagation()}
            />
          )}
          <Badge variant="secondary" className={TYPE_COLORS[record.type] || ""}>
            {record.type}
          </Badge>
          <span className="truncate font-mono text-sm">
            {record.name === "@" ? <span className="text-muted-foreground">@</span> : record.name}
          </span>
        </div>
        {!isSelectMode && (
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0" disabled={disabled}>
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onSelect={onEdit} disabled={disabled}>
                <Pencil className="mr-2 h-4 w-4" />
                {t("common.edit")}
              </DropdownMenuItem>
              <DropdownMenuItem
                onSelect={onDelete}
                disabled={disabled}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="mr-2 h-4 w-4" />
                {t("common.delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>

      {/* 第二行：value */}
      <div className="mt-2">
        <p className="break-all font-mono text-sm text-muted-foreground">
          {record.priority !== undefined && (
            <span className="mr-1 text-xs">[{record.priority}]</span>
          )}
          {record.value}
        </p>
      </div>

      {/* 第三行：ttl + proxy */}
      <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
        <span>TTL: {formatTTL(record.ttl)}</span>
        {showProxy && record.proxied !== undefined && (
          <span className="flex items-center gap-1">
            {record.proxied ? (
              <>
                <Shield className="h-3 w-3 text-orange-500" />
                <span>{t("dns.proxy")}</span>
              </>
            ) : (
              <ShieldOff className="h-3 w-3" />
            )}
          </span>
        )}
      </div>
    </Card>
  )
}
