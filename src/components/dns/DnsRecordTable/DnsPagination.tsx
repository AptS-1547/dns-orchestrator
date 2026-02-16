import { useTranslation } from "react-i18next"
import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNext,
  PaginationPrevious,
} from "@/components/ui/pagination"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { useIsMobile } from "@/hooks/useMediaQuery"
import { cn } from "@/lib/utils"

interface DnsPaginationProps {
  page: number
  pageSize: number
  totalCount: number
  onPageChange: (page: number) => void
  onPageSizeChange: (size: number) => void
}

export function DnsPagination({
  page,
  pageSize,
  totalCount,
  onPageChange,
  onPageSizeChange,
}: DnsPaginationProps) {
  const { t } = useTranslation()
  const isMobile = useIsMobile()
  const totalPages = Math.ceil(totalCount / pageSize)

  const renderDesktopPages = () => {
    const pages: (number | "ellipsis")[] = []

    if (totalPages <= 7) {
      for (let i = 1; i <= totalPages; i++) {
        pages.push(i)
      }
    } else {
      if (page <= 3) {
        pages.push(1, 2, 3, 4, "ellipsis", totalPages)
      } else if (page >= totalPages - 2) {
        pages.push(1, "ellipsis", totalPages - 3, totalPages - 2, totalPages - 1, totalPages)
      } else {
        pages.push(1, "ellipsis", page - 1, page, page + 1, "ellipsis", totalPages)
      }
    }

    return pages.map((p, i) =>
      p === "ellipsis" ? (
        // biome-ignore lint/suspicious/noArrayIndexKey: ellipsis items have no unique identifier
        <PaginationItem key={`ellipsis-${i}`}>
          <PaginationEllipsis className="h-8 w-8" />
        </PaginationItem>
      ) : (
        <PaginationItem key={p}>
          <PaginationLink
            onClick={() => onPageChange(p)}
            isActive={page === p}
            className="h-8 w-8 cursor-pointer text-xs"
          >
            {p}
          </PaginationLink>
        </PaginationItem>
      )
    )
  }

  return (
    <div className="flex items-center justify-between border-t px-4 py-2 md:justify-center">
      {/* 移动端分页选择器 */}
      <div className="flex items-center gap-1 md:hidden">
        <Select
          value={String(pageSize)}
          onValueChange={(val) => onPageSizeChange(Number(val))}
        >
          <SelectTrigger className="h-8 w-16">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {[10, 20, 50, 100].map((size) => (
              <SelectItem key={size} value={String(size)}>
                {size}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <span className="text-muted-foreground text-xs">{t("common.items")}</span>
      </div>

      <Pagination className="mx-0">
        <PaginationContent className="gap-1">
          <PaginationItem>
            <PaginationPrevious
              onClick={() => page > 1 && onPageChange(page - 1)}
              className={cn(
                "h-8 px-2 text-xs",
                page <= 1 ? "pointer-events-none opacity-50" : "cursor-pointer"
              )}
            />
          </PaginationItem>

          {isMobile ? (
            <PaginationItem>
              <Select
                value={String(page)}
                onValueChange={(val) => onPageChange(Number(val))}
              >
                <SelectTrigger className="h-8 w-auto gap-1 border-none bg-transparent px-2 shadow-none">
                  <span className="text-sm">
                    {page} / {totalPages}
                  </span>
                </SelectTrigger>
                <SelectContent className="max-h-[240px]">
                  {Array.from({ length: totalPages }, (_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: page numbers are stable and index-derived
                    <SelectItem key={i + 1} value={String(i + 1)}>
                      {t("common.pageWithNumber", { page: i + 1 })}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </PaginationItem>
          ) : (
            renderDesktopPages()
          )}

          <PaginationItem>
            <PaginationNext
              onClick={() => {
                if (page < totalPages) {
                  onPageChange(page + 1)
                }
              }}
              className={cn(
                "h-8 px-2 text-xs",
                page >= totalPages ? "pointer-events-none opacity-50" : "cursor-pointer"
              )}
            />
          </PaginationItem>
        </PaginationContent>
      </Pagination>
    </div>
  )
}
