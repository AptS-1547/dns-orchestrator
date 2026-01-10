import type { DnsRecordType } from "@/types"

/** 向导目的类型 */
export type WizardPurpose = "website" | "mail" | "subdomain" | "verification"

/** IP 类型 */
export type IpType = "ipv4" | "ipv6"

/** 向导状态 */
export interface WizardState {
  /** 目的 */
  purpose: WizardPurpose | null
  /** 子域名 */
  subdomain: string
  /** 目标地址（IP 或域名） */
  target: string
  /** IP 类型（仅 website 目的时使用） */
  ipType: IpType
  /** 邮件预设 ID（仅 mail 目的时使用） */
  mailPresetId: string | null
  /** TXT 值（仅 verification 目的时使用） */
  txtValue: string
}

/** 步骤 Props 基础类型 */
export interface StepProps {
  state: WizardState
  onStateChange: (updates: Partial<WizardState>) => void
  onNext: () => void
  onBack: () => void
  domainName: string
}

/** 从向导状态推断记录类型 */
export function getRecordTypeFromState(state: WizardState): DnsRecordType {
  switch (state.purpose) {
    case "website":
      return state.ipType === "ipv6" ? "AAAA" : "A"
    case "mail":
      return "MX"
    case "subdomain":
      return "CNAME"
    case "verification":
      return "TXT"
    default:
      return "A"
  }
}

/** 初始向导状态 */
export const initialWizardState: WizardState = {
  purpose: null,
  subdomain: "",
  target: "",
  ipType: "ipv4",
  mailPresetId: null,
  txtValue: "",
}
