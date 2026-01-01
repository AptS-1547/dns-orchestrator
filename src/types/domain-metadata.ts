/**
 * 域名元数据
 */
export interface DomainMetadata {
  /** 是否收藏 */
  isFavorite: boolean
  /** 标签列表（Phase 2） */
  tags: string[]
  /** 颜色标记（Phase 3） */
  color?: string
  /** 备注（Phase 3） */
  note?: string
  /** 最后修改时间（Unix 时间戳，毫秒） */
  updatedAt: number
}

/**
 * 域名元数据更新请求（部分更新，Phase 2/3 使用）
 */
export interface DomainMetadataUpdate {
  isFavorite?: boolean
  tags?: string[]
  /** null 表示清空字段 */
  color?: string | null
  /** null 表示清空字段 */
  note?: string | null
}
