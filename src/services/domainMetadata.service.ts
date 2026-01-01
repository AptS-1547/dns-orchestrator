import { transport } from "./transport"

class DomainMetadataService {
  /**
   * 获取域名元数据
   */
  async getMetadata(accountId: string, domainId: string) {
    return transport.invoke("get_domain_metadata", { accountId, domainId })
  }

  /**
   * 切换收藏状态
   * @returns 新的收藏状态
   */
  async toggleFavorite(accountId: string, domainId: string) {
    return transport.invoke("toggle_domain_favorite", { accountId, domainId })
  }

  /**
   * 获取账户下的收藏域名 ID 列表
   */
  async listAccountFavorites(accountId: string) {
    return transport.invoke("list_account_favorite_domain_keys", { accountId })
  }
}

export const domainMetadataService = new DomainMetadataService()
