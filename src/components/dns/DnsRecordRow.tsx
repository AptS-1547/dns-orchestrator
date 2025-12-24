import { MoreHorizontal, Pencil, Shield, ShieldOff, Trash2 } from "lucide-react"
import { memo } from "react"
import { useTranslation } from "react-i18next"
import { CopyableText } from "@/components/toolbox/shared/CopyableText"
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
  domainName: string
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

export const DnsRecordRow = memo(function DnsRecordRow({
  record,
  domainName,
  onEdit,
  onDelete,
  disabled = false,
  showProxy = false,
  asFragment = false,
}: DnsRecordRowProps) {
  const { t } = useTranslation()
  // 计算完整域名：@ 表示根域名，否则是 name.domain
  const fullDomain = record.name === "@" ? domainName : `${record.name}.${domainName}`
  const cells = (
    <>
      <TableCell>
        <Badge variant="secondary" className={TYPE_COLORS[record.type] || ""}>
          {record.type}
        </Badge>
      </TableCell>
      <TableCell className="font-mono text-sm">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="block max-w-xs truncate">
                <CopyableText value={fullDomain}>
                  {record.name === "@" ? <span className="text-muted-foreground">@</span> : record.name}
                </CopyableText>
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p className="font-mono text-xs">{fullDomain}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </TableCell>
      <TableCell>
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <span className="block max-w-xs truncate font-mono text-sm">
                {record.priority !== undefined && (
                  <span className="mr-2 text-muted-foreground">[{record.priority}]</span>
                )}
                <CopyableText value={record.value}>{record.value}</CopyableText>
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-md break-all font-mono text-xs">{record.value}</p>
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
