//! 导入导出相关类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use dns_orchestrator_provider::ProviderType;

/// 单个账号的导出数据（包含凭证）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedAccount {
    /// 账户 ID
    pub id: String,
    /// 账户名称
    pub name: String,
    /// DNS 服务商类型
    pub provider: ProviderType,
    /// 创建时间
    pub created_at: String,
    /// 更新时间
    pub updated_at: String,
    /// 凭证数据
    pub credentials: HashMap<String, String>,
}

/// 导出文件头部（明文部分）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFileHeader {
    /// 文件格式版本
    pub version: u32,
    /// 是否加密
    pub encrypted: bool,
    /// 加密时使用的盐值（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    /// 加密时使用的 IV/Nonce（Base64 编码）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    /// 导出时间
    pub exported_at: String,
    /// 应用版本
    pub app_version: String,
}

/// 完整的导出文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    /// 文件头部
    pub header: ExportFileHeader,
    /// 账号数据（加密时为 Base64 编码的密文，未加密时为 JSON 数组）
    pub data: serde_json::Value,
}

/// 导出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsRequest {
    /// 要导出的账号 ID 列表
    pub account_ids: Vec<String>,
    /// 是否加密
    pub encrypt: bool,
    /// 加密密码（仅当 encrypt=true 时需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// 导出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsResponse {
    /// 导出的 JSON 内容
    pub content: String,
    /// 建议的文件名
    pub suggested_filename: String,
}

/// 导入请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAccountsRequest {
    /// 导入文件的内容
    pub content: String,
    /// 解密密码（如果文件加密）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// 导入预览（用于显示将要导入的账号）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    /// 文件是否加密
    pub encrypted: bool,
    /// 账号数量
    pub account_count: usize,
    /// 账号预览列表（仅在未加密或已解密后可用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<ImportPreviewAccount>>,
}

/// 导入预览中的账号信息（不含敏感凭证）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewAccount {
    /// 账户名称
    pub name: String,
    /// DNS 服务商类型
    pub provider: ProviderType,
    /// 是否与现有账号名称冲突
    pub has_conflict: bool,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    /// 成功导入的账号数量
    pub success_count: usize,
    /// 失败的账号及原因
    pub failures: Vec<ImportFailure>,
}

/// 导入失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportFailure {
    /// 账户名称
    pub name: String,
    /// 失败原因
    pub reason: String,
}
