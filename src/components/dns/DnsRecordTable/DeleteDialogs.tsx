import { Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
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
import type { DnsRecord } from "@/types"

interface DeleteRecordDialogProps {
  record: DnsRecord | null
  isDeleting: boolean
  onConfirm: () => void
  onClose: () => void
}

export function DeleteRecordDialog({
  record,
  isDeleting,
  onConfirm,
  onClose,
}: DeleteRecordDialogProps) {
  const { t } = useTranslation()

  return (
    <AlertDialog open={!!record} onOpenChange={(open) => !open && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("dns.deleteConfirm")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("dns.deleteConfirmDesc", { name: record?.name })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>{t("common.cancel")}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            disabled={isDeleting}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {isDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t("common.delete")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

interface BatchDeleteDialogProps {
  open: boolean
  selectedCount: number
  isDeleting: boolean
  onConfirm: () => void
  onClose: () => void
}

export function BatchDeleteDialog({
  open,
  selectedCount,
  isDeleting,
  onConfirm,
  onClose,
}: BatchDeleteDialogProps) {
  const { t } = useTranslation()

  return (
    <AlertDialog open={open} onOpenChange={(o) => !o && onClose()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("dns.batchDeleteConfirm")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t("dns.batchDeleteConfirmDesc", { count: selectedCount })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>{t("common.cancel")}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            disabled={isDeleting}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
          >
            {isDeleting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {t("common.delete")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
