import type { DnsRecordType } from "@/types"
import { RECORD_TYPE_INFO } from "@/types/dns"

export interface RecordFieldConfig {
  key: string
  labelKey: string
  type: "text" | "number" | "select"
  required?: boolean
  min?: number
  max?: number
  placeholder?: string
  selectOptions?: { value: string; label: string }[]
}

export interface RecordTypeConfig {
  fields: RecordFieldConfig[]
  defaults: Record<string, string | number>
  primaryValueKey: string
}

export const RECORD_TYPE_CONFIGS: Record<DnsRecordType, RecordTypeConfig> = {
  A: {
    fields: [
      {
        key: "address",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.A.example,
      },
    ],
    defaults: { address: "" },
    primaryValueKey: "address",
  },
  AAAA: {
    fields: [
      {
        key: "address",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.AAAA.example,
      },
    ],
    defaults: { address: "" },
    primaryValueKey: "address",
  },
  CNAME: {
    fields: [
      {
        key: "target",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.CNAME.example,
      },
    ],
    defaults: { target: "" },
    primaryValueKey: "target",
  },
  MX: {
    fields: [
      {
        key: "priority",
        labelKey: "dns.priority",
        type: "number",
        required: true,
        min: 0,
        max: 65535,
        placeholder: "10",
      },
      {
        key: "exchange",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.MX.example,
      },
    ],
    defaults: { priority: 10, exchange: "" },
    primaryValueKey: "exchange",
  },
  TXT: {
    fields: [
      {
        key: "text",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.TXT.example,
      },
    ],
    defaults: { text: "" },
    primaryValueKey: "text",
  },
  NS: {
    fields: [
      {
        key: "nameserver",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.NS.example,
      },
    ],
    defaults: { nameserver: "" },
    primaryValueKey: "nameserver",
  },
  SRV: {
    fields: [
      {
        key: "priority",
        labelKey: "dns.priority",
        type: "number",
        required: true,
        min: 0,
        max: 65535,
        placeholder: "10",
      },
      {
        key: "weight",
        labelKey: "dns.weight",
        type: "number",
        required: true,
        min: 0,
        max: 65535,
        placeholder: "5",
      },
      {
        key: "port",
        labelKey: "dns.port",
        type: "number",
        required: true,
        min: 0,
        max: 65535,
        placeholder: "80",
      },
      {
        key: "target",
        labelKey: "dns.target",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.SRV.example,
      },
    ],
    defaults: { priority: 10, weight: 5, port: 80, target: "" },
    primaryValueKey: "target",
  },
  CAA: {
    fields: [
      {
        key: "flags",
        labelKey: "dns.flags",
        type: "number",
        required: true,
        min: 0,
        max: 255,
        placeholder: "0",
      },
      {
        key: "tag",
        labelKey: "dns.tag",
        type: "select",
        selectOptions: [
          { value: "issue", label: "issue" },
          { value: "issuewild", label: "issuewild" },
          { value: "iodef", label: "iodef" },
        ],
      },
      {
        key: "value",
        labelKey: "dns.value",
        type: "text",
        required: true,
        placeholder: RECORD_TYPE_INFO.CAA.example,
      },
    ],
    defaults: { flags: 0, tag: "issue", value: "" },
    primaryValueKey: "value",
  },
}
