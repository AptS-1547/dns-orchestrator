/**
 * DNS 记录服务
 */

import type {
  ApiResponse,
  BatchDeleteRequest,
  BatchDeleteResult,
  CreateDnsRecordRequest,
  DnsRecord,
  PaginatedResponse,
  UpdateDnsRecordRequest,
} from "@/types"
import { transport } from "./transport"

export interface ListDnsRecordsParams {
  accountId: string
  domainId: string
  page?: number
  pageSize?: number
  keyword?: string | null
  recordType?: string | null
}

class DnsService {
  listRecords(params: ListDnsRecordsParams): Promise<ApiResponse<PaginatedResponse<DnsRecord>>> {
    return transport.invoke("list_dns_records", params)
  }

  createRecord(
    accountId: string,
    request: CreateDnsRecordRequest
  ): Promise<ApiResponse<DnsRecord>> {
    return transport.invoke("create_dns_record", { accountId, request })
  }

  updateRecord(
    accountId: string,
    recordId: string,
    request: UpdateDnsRecordRequest
  ): Promise<ApiResponse<DnsRecord>> {
    return transport.invoke("update_dns_record", { accountId, recordId, request })
  }

  deleteRecord(accountId: string, recordId: string, domainId: string): Promise<ApiResponse<void>> {
    return transport.invoke("delete_dns_record", { accountId, recordId, domainId })
  }

  batchDeleteRecords(
    accountId: string,
    request: BatchDeleteRequest
  ): Promise<ApiResponse<BatchDeleteResult>> {
    return transport.invoke("batch_delete_dns_records", { accountId, request })
  }
}

export const dnsService = new DnsService()
