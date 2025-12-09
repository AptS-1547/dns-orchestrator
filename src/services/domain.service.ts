/**
 * 域名服务
 */

import type { ApiResponse, Domain, PaginatedResponse } from "@/types"
import { transport } from "./transport"

class DomainService {
  listDomains(
    accountId: string,
    page?: number,
    pageSize?: number
  ): Promise<ApiResponse<PaginatedResponse<Domain>>> {
    return transport.invoke("list_domains", { accountId, page, pageSize })
  }

  getDomain(accountId: string, domainId: string): Promise<ApiResponse<Domain>> {
    return transport.invoke("get_domain", { accountId, domainId })
  }
}

export const domainService = new DomainService()
