import type { Domain } from "@/types"

export interface DomainItemBaseProps {
  domain: Domain
  accountId: string
  isBatchMode: boolean
  isSelected: boolean
  isDark: boolean
  onClick?: () => void
  onClickTag?: (tag: string) => void
}

export interface DomainItemProps extends Omit<DomainItemBaseProps, "onClick"> {
  onSelect: () => void
  onToggleSelection: () => void
}
