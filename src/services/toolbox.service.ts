import type {
  ApiResponse,
  DnsLookupResult,
  DnsPropagationResult,
  DnssecResult,
  HttpHeaderCheckRequest,
  HttpHeaderCheckResult,
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

  httpHeaderCheck(request: HttpHeaderCheckRequest): Promise<ApiResponse<HttpHeaderCheckResult>> {
    return transport.invoke("http_header_check", { request })
  }

  dnsPropagationCheck(
    domain: string,
    recordType: string
  ): Promise<ApiResponse<DnsPropagationResult>> {
    return transport.invoke("dns_propagation_check", { domain, recordType })
  }

  dnssecCheck(domain: string, nameserver: string | null): Promise<ApiResponse<DnssecResult>> {
    return transport.invoke("dnssec_check", { domain, nameserver })
  }
}

export const toolboxService = new ToolboxService()
