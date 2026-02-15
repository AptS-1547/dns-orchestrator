use serde::{Deserialize, Serialize};

// Re-export Core 应用层类型
pub use dns_orchestrator_core::types::{
    Account, AppDomain, BatchDeleteRequest, BatchTagRequest, BatchTagResult, CreateAccountRequest,
    DomainMetadata, DomainMetadataUpdate, ExportAccountsRequest, ExportAccountsResponse,
    ImportAccountsRequest, ImportPreview, ImportResult, UpdateAccountRequest,
};

// Re-export Provider 类型（通过 core re-export）
pub use dns_orchestrator_core::types::{
    BatchDeleteResult, CreateDnsRecordRequest, DnsRecord, DnsRecordType, PaginatedResponse,
    ProviderMetadata, UpdateDnsRecordRequest,
};

/// Domain 别名（保持命令层命名习惯）
pub type Domain = AppDomain;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success_construction() {
        let resp = ApiResponse::success(42);
        assert!(resp.success);
        assert_eq!(resp.data, Some(42));
    }

    #[test]
    fn test_api_response_serialize() {
        let resp = ApiResponse::success("hello");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["data"], "hello");
    }

    #[test]
    fn test_api_response_roundtrip() {
        let resp = ApiResponse::success(vec![1, 2, 3]);
        let json_str = serde_json::to_string(&resp).unwrap();
        let deserialized: ApiResponse<Vec<i32>> = serde_json::from_str(&json_str).unwrap();
        assert!(deserialized.success);
        assert_eq!(deserialized.data, Some(vec![1, 2, 3]));
    }
}
