import type { DomainStatus } from "@/types"

export const statusConfig: Record<
  DomainStatus,
  { labelKey: string; variant: "default" | "secondary" | "destructive" | "outline" }
> = {
  active: { labelKey: "domain.status.active", variant: "default" },
  paused: { labelKey: "domain.status.paused", variant: "secondary" },
  pending: { labelKey: "domain.status.pending", variant: "outline" },
  error: { labelKey: "domain.status.error", variant: "destructive" },
  unknown: { labelKey: "domain.status.unknown", variant: "outline" },
}
