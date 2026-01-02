import type { BatchTagRequest, DomainMetadataUpdate } from "@/types/domain-metadata"
import { transport } from "./transport"

class DomainMetadataService {
  /**
   * 获取域名元数据
   */
  async getMetadata(accountId: string, domainId: string) {
    return transport.invoke("get_domain_metadata", { accountId, domainId })
  }

  /**
   * 更新域名元数据（通用部分更新，Phase 3）
   * @returns 更新后的完整元数据
   */
  async updateMetadata(accountId: string, domainId: string, update: DomainMetadataUpdate) {
    return transport.invoke("update_domain_metadata", { accountId, domainId, update })
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

  /**
   * 添加标签
   * @returns 更新后的标签列表
   */
  async addTag(accountId: string, domainId: string, tag: string) {
    return transport.invoke("add_domain_tag", { accountId, domainId, tag })
  }

  /**
   * 移除标签
   * @returns 更新后的标签列表
   */
  async removeTag(accountId: string, domainId: string, tag: string) {
    return transport.invoke("remove_domain_tag", { accountId, domainId, tag })
  }

  /**
   * 批量设置标签
   * @returns 更新后的标签列表
   */
  async setTags(accountId: string, domainId: string, tags: string[]) {
    return transport.invoke("set_domain_tags", { accountId, domainId, tags })
  }

  /**
   * 按标签查询域名
   * @returns 域名键列表（格式: account_id::domain_id）
   */
  async findByTag(tag: string) {
    return transport.invoke("find_domains_by_tag", { tag })
  }

  /**
   * 获取所有标签（用于自动补全）
   */
  async listAllTags() {
    return transport.invoke("list_all_domain_tags")
  }

  /**
   * 批量添加标签
   */
  async batchAddTags(requests: BatchTagRequest[]) {
    return transport.invoke("batch_add_domain_tags", { requests })
  }

  /**
   * 批量移除标签
   */
  async batchRemoveTags(requests: BatchTagRequest[]) {
    return transport.invoke("batch_remove_domain_tags", { requests })
  }

  /**
   * 批量替换标签
   */
  async batchSetTags(requests: BatchTagRequest[]) {
    return transport.invoke("batch_set_domain_tags", { requests })
  }
}

export const domainMetadataService = new DomainMetadataService()
