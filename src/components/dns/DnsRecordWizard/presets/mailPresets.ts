import type { RecordData } from "@/types"

/** 邮件服务预设记录 */
export interface MailPresetRecord {
  type: "MX" | "TXT"
  name: string // @ 表示根域名
  priority?: number // 仅 MX 记录需要
  value: string
  ttl: number
}

/** 邮件服务预设 */
export interface MailPreset {
  id: string
  nameKey: string // i18n key
  descriptionKey?: string // i18n key
  records: MailPresetRecord[]
}

/** 所有支持的邮件服务预设 */
export const MAIL_PRESETS: MailPreset[] = [
  {
    id: "gmail",
    nameKey: "dns.wizard.mail.presets.gmail",
    descriptionKey: "dns.wizard.mail.presets.gmailDesc",
    records: [{ type: "MX", name: "@", priority: 1, value: "smtp.google.com", ttl: 600 }],
  },
  {
    id: "microsoft365",
    nameKey: "dns.wizard.mail.presets.microsoft365",
    descriptionKey: "dns.wizard.mail.presets.microsoft365Desc",
    records: [
      // Microsoft 365 MX 记录格式: <domain>-<domain>.mail.protection.outlook.com
      // 用户需要替换为自己的域名，这里用占位符
      {
        type: "MX",
        name: "@",
        priority: 0,
        value: "{{domain}}.mail.protection.outlook.com",
        ttl: 600,
      },
    ],
  },
  {
    id: "aliyun",
    nameKey: "dns.wizard.mail.presets.aliyun",
    descriptionKey: "dns.wizard.mail.presets.aliyunDesc",
    records: [
      { type: "MX", name: "@", priority: 5, value: "mx1.qiye.aliyun.com", ttl: 600 },
      { type: "MX", name: "@", priority: 10, value: "mx2.qiye.aliyun.com", ttl: 600 },
    ],
  },
  {
    id: "tencent",
    nameKey: "dns.wizard.mail.presets.tencent",
    descriptionKey: "dns.wizard.mail.presets.tencentDesc",
    records: [
      { type: "MX", name: "@", priority: 5, value: "mxbiz1.qq.com", ttl: 600 },
      { type: "MX", name: "@", priority: 10, value: "mxbiz2.qq.com", ttl: 600 },
    ],
  },
  {
    id: "netease",
    nameKey: "dns.wizard.mail.presets.netease",
    descriptionKey: "dns.wizard.mail.presets.neteaseDesc",
    records: [
      { type: "MX", name: "@", priority: 5, value: "qiye163mx01.mxmail.netease.com", ttl: 600 },
      { type: "MX", name: "@", priority: 10, value: "qiye163mx02.mxmail.netease.com", ttl: 600 },
    ],
  },
]

/** 将预设记录转换为 RecordData 格式 */
export function presetRecordToRecordData(
  record: MailPresetRecord,
  domainName?: string
): RecordData {
  let value = record.value
  // 处理 Microsoft 365 的域名占位符
  if (domainName && value.includes("{{domain}}")) {
    // example.com -> example-com
    const sanitized = domainName.replace(/\./g, "-")
    value = value.replace("{{domain}}", sanitized)
  }

  if (record.type === "MX") {
    return {
      type: "MX",
      content: {
        priority: record.priority ?? 10,
        exchange: value,
      },
    }
  }
  return {
    type: "TXT",
    content: {
      text: value,
    },
  }
}

/** 根据 ID 获取预设 */
export function getMailPresetById(id: string): MailPreset | undefined {
  return MAIL_PRESETS.find((p) => p.id === id)
}
