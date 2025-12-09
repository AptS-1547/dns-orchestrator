/**
 * 工具箱服务
 */

import type {
  ApiResponse,
  DnsLookupResult,
  IpLookupResult,
  SslCheckResult,
  WhoisResult,
} from "@/types"
import { transport } from "./transport"

class ToolboxService {
  whoisLookup(domain: string): Promise<ApiResponse<WhoisResult>> {
    return transport.invoke("whois_lookup", { domain })
  }

  dnsLookup(
    domain: string,
    recordType: string,
    nameserver: string | null
  ): Promise<ApiResponse<DnsLookupResult>> {
    return transport.invoke("dns_lookup", { domain, recordType, nameserver })
  }

  ipLookup(query: string): Promise<ApiResponse<IpLookupResult>> {
    return transport.invoke("ip_lookup", { query })
  }

  sslCheck(domain: string, port?: number): Promise<ApiResponse<SslCheckResult>> {
    return transport.invoke("ssl_check", { domain, port })
  }
}

export const toolboxService = new ToolboxService()
