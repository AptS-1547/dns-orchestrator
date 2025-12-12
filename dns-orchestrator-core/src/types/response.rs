//! API 响应相关类型定义

use serde::{Deserialize, Serialize};

/// API 响应包装类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 是否成功
    pub success: bool,
    /// 响应数据
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    #[must_use]
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
        }
    }
}

/// 批量删除 DNS 记录请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    /// 域名 ID
    pub domain_id: String,
    /// 记录 ID 列表
    pub record_ids: Vec<String>,
}

/// 批量删除结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResult {
    /// 成功删除的数量
    pub success_count: usize,
    /// 失败的数量
    pub failed_count: usize,
    /// 失败详情
    pub failures: Vec<BatchDeleteFailure>,
}

/// 批量删除失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteFailure {
    /// 记录 ID
    pub record_id: String,
    /// 失败原因
    pub reason: String,
}
