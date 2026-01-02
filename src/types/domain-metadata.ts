/**
 * 域名元数据
 */
export interface DomainMetadata {
  /** 是否收藏 */
  isFavorite: boolean
  /** 标签列表（Phase 2） */
  tags: string[]
  /** 颜色标记（Phase 3，"none" 表示无颜色） */
  color: string
  /** 备注（Phase 3） */
  note?: string
  /** 收藏时间（ISO8601 字符串） */
  favoritedAt?: string
  /** 最后修改时间（ISO8601 字符串） */
  updatedAt: string
}

/**
 * 域名元数据更新请求（部分更新，Phase 2/3 使用）
 */
export interface DomainMetadataUpdate {
  isFavorite?: boolean
  tags?: string[]
  /** "none" 表示清空颜色 */
  color?: string
  /** null 表示清空字段 */
  note?: string | null
}

/**
 * 批量标签操作请求
 */
export interface BatchTagRequest {
  accountId: string
  domainId: string
  tags: string[]
}

/**
 * 批量标签操作结果
 */
export interface BatchTagResult {
  successCount: number
  failedCount: number
  failures: BatchTagFailure[]
}

/**
 * 批量标签操作失败详情
 */
export interface BatchTagFailure {
  accountId: string
  domainId: string
  reason: string
}
