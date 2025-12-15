//! 阿里云 HTTP 请求方法（重构版：使用通用 HTTP 工具）

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::http_client::HttpUtils;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::{
    ALIYUN_DNS_HOST, ALIYUN_DNS_VERSION, AliyunProvider, AliyunResponse, EMPTY_BODY_SHA256,
    serialize_to_query_string,
};

impl AliyunProvider {
    /// 执行阿里云 API 请求 (RPC 风格: 参数通过 query string 传递)
    pub(crate) async fn request<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        action: &str,
        params: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        // 1. 序列化参数为 query string
        let query_string = serialize_to_query_string(params)?;

        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let nonce = uuid::Uuid::new_v4().to_string();

        // 2. 生成签名 (使用 query string)
        let authorization = self.sign(action, &query_string, &timestamp, &nonce);

        // 3. 构造 URL (参数在 query string 中)
        let url = if query_string.is_empty() {
            format!("https://{ALIYUN_DNS_HOST}/")
        } else {
            format!("https://{ALIYUN_DNS_HOST}/?{query_string}")
        };

        // 4. 发送请求 (body 为空，使用 HttpUtils)
        let request = self
            .client
            .post(&url)
            .header("Host", ALIYUN_DNS_HOST)
            .header("x-acs-action", action)
            .header("x-acs-version", ALIYUN_DNS_VERSION)
            .header("x-acs-date", &timestamp)
            .header("x-acs-signature-nonce", &nonce)
            .header("x-acs-content-sha256", EMPTY_BODY_SHA256)
            .header("Authorization", authorization);

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "POST",
            &format!("{} (Action: {})", url, action),
            self.max_retries,
        )
        .await?;

        // 5. 先检查是否有错误响应
        if let Ok(error_response) = serde_json::from_str::<AliyunResponse<()>>(&response_text)
            && let (Some(code), Some(message)) = (error_response.code, error_response.message)
        {
            log::error!("API 错误: {code} - {message}");
            return Err(self.map_error(RawApiError::with_code(&code, &message), ctx));
        }

        // 6. 解析成功响应
        HttpUtils::parse_json(&response_text, self.provider_name())
    }
}
