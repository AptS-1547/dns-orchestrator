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
import { MoreHorizontal, Pencil, Shield, ShieldOff, Trash2 } from "lucide-react"

interface DnsRecordRowProps {
  record: DnsRecord
  onEdit: () => void
  onDelete: () => void
  disabled?: boolean
  showProxy?: boolean
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

export function DnsRecordRow({
  record,
  onEdit,
  onDelete,
  disabled = false,
  showProxy = false,
}: DnsRecordRowProps) {
  return (
    <TableRow>
      <TableCell>
        <Badge variant="secondary" className={TYPE_COLORS[record.type] || ""}>
          {record.type}
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
                {record.priority !== undefined && (
                  <span className="mr-2 text-muted-foreground">[{record.priority}]</span>
                )}
                {record.value}
              </span>
            </TooltipTrigger>
            <TooltipContent>
              <p className="max-w-md break-all font-mono text-xs">{record.value}</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </TableCell>
      <TableCell className="text-muted-foreground text-sm">{formatTTL(record.ttl)}</TableCell>
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
              编辑
            </DropdownMenuItem>
            <DropdownMenuItem
              onSelect={onDelete}
              disabled={disabled}
              className="text-destructive focus:text-destructive"
            >
              <Trash2 className="mr-2 h-4 w-4" />
              删除
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </TableCell>
    </TableRow>
  )
}
