import { MoreHorizontal, Pencil, Shield, ShieldOff, Trash2 } from "lucide-react"
import { memo } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { TableCell, TableRow } from "@/components/ui/table"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import type { DnsRecord } from "@/types"

interface DnsRecordRowProps {
  record: DnsRecord
  onEdit: () => void
  onDelete: () => void
  disabled?: boolean
  showProxy?: boolean
  /** 作为 Fragment 渲染（不包含 TableRow，用于外部添加 checkbox） */
  asFragment?: boolean
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

function formatTTL(
  ttl: number,
  t: (key: string, options?: Record<string, unknown>) => string
): string {
  if (ttl === 1) return t("dns.ttlAuto")
  if (ttl < 60) return t("dns.ttlSeconds", { count: ttl })
  if (ttl < 3600) return t("dns.ttlMinutes", { count: Math.floor(ttl / 60) })
  if (ttl < 86400) return t("dns.ttlHours", { count: Math.floor(ttl / 3600) })
  return t("dns.ttlDay")
}

/** 渲染记录的值显示 */
function renderRecordValue(record: DnsRecord) {
  const { data } = record

  switch (data.type) {
    case "A":
    case "AAAA":
      return data.content.address
    case "CNAME":
      return data.content.target
    case "MX":
      return (
        <>
          <span className="mr-2 inline-flex items-center rounded-full border border-violet-400 bg-violet-100 px-2 py-0.5 font-medium text-violet-700 text-xs dark:border-violet-500 dark:bg-violet-900/50 dark:text-violet-300">
            {data.content.priority}
          </span>
          {data.content.exchange}
        </>
      )
    case "TXT":
      return data.content.text
    case "NS":
      return data.content.nameserver
    case "SRV":
      return (
        <>
          <span className="mr-2 inline-flex items-center rounded-full border border-violet-400 bg-violet-100 px-2 py-0.5 font-medium text-violet-700 text-xs dark:border-violet-500 dark:bg-violet-900/50 dark:text-violet-300">
            {data.content.priority}
          </span>
          <span className="mr-2 inline-flex items-center rounded-full border border-blue-400 bg-blue-100 px-2 py-0.5 font-medium text-blue-700 text-xs dark:border-blue-500 dark:bg-blue-900/50 dark:text-blue-300">
            {data.content.weight}
          </span>
          <span className="mr-2 inline-flex items-center rounded-full border border-green-400 bg-green-100 px-2 py-0.5 font-medium text-green-700 text-xs dark:border-green-500 dark:bg-green-900/50 dark:text-green-300">
            {data.content.port}
          </span>
          {data.content.target}
        </>
      )
    case "CAA":
      return (
        <>
          <span className="mr-2 inline-flex items-center rounded-full border border-red-400 bg-red-100 px-2 py-0.5 font-medium text-red-700 text-xs dark:border-red-500 dark:bg-red-900/50 dark:text-red-300">
            {data.content.flags}
          </span>
          <span className="mr-2 inline-flex items-center rounded-full border border-orange-400 bg-orange-100 px-2 py-0.5 font-medium text-orange-700 text-xs dark:border-orange-500 dark:bg-orange-900/50 dark:text-orange-300">
            {data.content.tag}
          </span>
          {data.content.value}
        </>
      )
  }
}

/** 获取显示用的纯文本值（用于 tooltip）*/
function getDisplayText(record: DnsRecord): string {
  const { data } = record

  switch (data.type) {
    case "A":
    case "AAAA":
      return data.content.address
    case "CNAME":
      return data.content.target
    case "MX":
      return `[${data.content.priority}] ${data.content.exchange}`
    case "TXT":
      return data.content.text
    case "NS":
      return data.content.nameserver
    case "SRV":
      return `[${data.content.priority}] [${data.content.weight}] [${data.content.port}] ${data.content.target}`
    case "CAA":
      return `[${data.content.flags}] [${data.content.tag}] ${data.content.value}`
  }
}

export const DnsRecordRow = memo(function DnsRecordRow({
  record,
  onEdit,
  onDelete,
  disabled = false,
  showProxy = false,
  asFragment = false,
}: DnsRecordRowProps) {
  const { t } = useTranslation()
  const cells = (
    <>
      <TableCell>
        <Badge variant="secondary" className={TYPE_COLORS[record.data.type] || ""}>
          {record.data.type}
        </Badge>
      </TableCell>
      <TableCell className="font-mono text-sm">
        {record.name === "@" ? <span className="text-muted-foreground">@</span> : record.name}
      </TableCell>
      <TableCell>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="block max-w-xs truncate font-mono text-sm">
                {renderRecordValue(record)}
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-md break-all font-mono text-xs">{getDisplayText(record)}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </TableCell>
      <TableCell className="text-muted-foreground text-sm">{formatTTL(record.ttl, t)}</TableCell>
      {showProxy && (
        <TableCell>
          {record.proxied !== undefined &&
            (record.proxied ? (
              <Shield className="h-4 w-4 text-orange-500" />
            ) : (
              <ShieldOff className="h-4 w-4 text-muted-foreground" />
            ))}
        </TableCell>
      )}
      <TableCell className="text-right">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="h-8 w-8" disabled={disabled}>
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
      </TableCell>
    </>
  )

  if (asFragment) {
    return cells
  }

  return <TableRow>{cells}</TableRow>
})
