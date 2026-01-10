import { useIsMobile } from "@/hooks/useMediaQuery"
import { DomainItemDesktop } from "./DomainItemDesktop"
import { DomainItemMobile } from "./DomainItemMobile"
import type { DomainItemProps } from "./types"

export function DomainItem({
  domain,
  accountId,
  isBatchMode,
  isSelected,
  isDark,
  onSelect,
  onToggleSelection,
  onClickTag,
}: DomainItemProps) {
  const isMobile = useIsMobile()

  const handleClick = () => {
    if (isBatchMode) {
      onToggleSelection()
    } else {
      onSelect()
    }
  }

  const baseProps = {
    domain,
    accountId,
    isBatchMode,
    isSelected,
    isDark,
    onClick: handleClick,
    onClickTag,
  }

  if (isMobile) {
    return <DomainItemMobile {...baseProps} />
  }

  return <DomainItemDesktop {...baseProps} />
}

export { statusConfig } from "./constants"
export type { DomainItemBaseProps, DomainItemProps } from "./types"
