//! 域名元数据类型定义

use serde::{Deserialize, Serialize};

/// 默认颜色值（无颜色）
fn default_color() -> String {
    "none".to_string()
}

/// 域名元数据键（复合主键）
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataKey {
    pub account_id: String,
    pub domain_id: String,
}

impl DomainMetadataKey {
    /// 创建新的元数据键
    #[must_use]
    pub fn new(account_id: String, domain_id: String) -> Self {
        Self {
            account_id,
            domain_id,
        }
    }

    /// 生成存储用的字符串键（格式: `account_id::domain_id`）
    #[must_use]
    pub fn to_storage_key(&self) -> String {
        format!("{}::{}", self.account_id, self.domain_id)
    }

    /// 从存储键解析
    #[must_use]
    pub fn from_storage_key(key: &str) -> Option<Self> {
        let parts: Vec<&str> = key.split("::").collect();
        if parts.len() != 2 {
            return None;
        }
        Some(Self {
            account_id: parts[0].to_string(),
            domain_id: parts[1].to_string(),
        })
    }
}

/// 域名元数据
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    /// 是否收藏
    #[serde(default)]
    pub is_favorite: bool,

    /// 标签列表（Phase 2 实现）
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// 颜色标记（"none" 表示无颜色，Phase 3 实现）
    #[serde(default = "default_color")]
    pub color: String,

    /// 备注（可选，Phase 3 实现）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// 收藏时间（仅收藏时有值）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorited_at: Option<chrono::DateTime<chrono::Utc>>,

    /// 最后修改时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for DomainMetadata {
    fn default() -> Self {
        Self {
            is_favorite: false,
            tags: Vec::new(),
            color: "none".to_string(),
            note: None,
            favorited_at: None,
            updated_at: chrono::Utc::now(),
        }
    }
}

impl DomainMetadata {
    /// 创建新的元数据（全部字段）
    #[must_use]
    pub fn new(
        is_favorite: bool,
        tags: Vec<String>,
        color: String,
        note: Option<String>,
        favorited_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Self {
        Self {
            is_favorite,
            tags,
            color,
            note,
            favorited_at,
            updated_at: chrono::Utc::now(),
        }
    }

    /// 刷新更新时间
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// 是否为空元数据（所有字段都是默认值）
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_favorite
            && self.tags.is_empty()
            && self.color == "none"
            && self.note.is_none()
            && self.favorited_at.is_none()
    }
}

/// 域名元数据更新请求（支持部分更新）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// 空字符串表示清空颜色
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Option<String>>,
}

impl DomainMetadataUpdate {
    /// 应用更新到现有元数据
    pub fn apply_to(&self, metadata: &mut DomainMetadata) {
        if let Some(is_favorite) = self.is_favorite {
            metadata.is_favorite = is_favorite;
        }
        if let Some(ref tags) = self.tags {
            metadata.tags = tags.clone();
        }
        if let Some(ref color) = self.color {
            metadata.color = color.clone();
        }
        if let Some(ref note) = self.note {
            metadata.note = note.clone();
        }
        metadata.touch();
    }
}

/// 批量标签操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagRequest {
    pub account_id: String,
    pub domain_id: String,
    pub tags: Vec<String>,
}

/// 批量标签操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub failures: Vec<BatchTagFailure>,
}

/// 批量标签操作失败详情
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagFailure {
    pub account_id: String,
    pub domain_id: String,
    pub reason: String,
}
