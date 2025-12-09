/**
 * 账户服务
 */

import type {
  Account,
  ApiResponse,
  CreateAccountRequest,
  ExportAccountsRequest,
  ExportAccountsResponse,
  ImportAccountsRequest,
  ImportPreview,
  ImportResult,
  ProviderInfo,
} from "@/types"
import { transport } from "./transport"

class AccountService {
  listAccounts(): Promise<ApiResponse<Account[]>> {
    return transport.invoke("list_accounts")
  }

  createAccount(request: CreateAccountRequest): Promise<ApiResponse<Account>> {
    return transport.invoke("create_account", { request })
  }

  deleteAccount(accountId: string): Promise<ApiResponse<void>> {
    return transport.invoke("delete_account", { accountId })
  }

  listProviders(): Promise<ApiResponse<ProviderInfo[]>> {
    return transport.invoke("list_providers")
  }

  exportAccounts(request: ExportAccountsRequest): Promise<ApiResponse<ExportAccountsResponse>> {
    return transport.invoke("export_accounts", { request })
  }

  previewImport(content: string, password: string | null): Promise<ApiResponse<ImportPreview>> {
    return transport.invoke("preview_import", { content, password })
  }

  importAccounts(request: ImportAccountsRequest): Promise<ApiResponse<ImportResult>> {
    return transport.invoke("import_accounts", { request })
  }
}

export const accountService = new AccountService()
