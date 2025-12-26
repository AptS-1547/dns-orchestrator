//! 华为云 HTTP 请求方法（重构版：消除代码重复）

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, Result};
use crate::http_client::HttpUtils;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::types::ErrorResponse;
use super::{HUAWEICLOUD_DNS_HOST, HuaweicloudProvider};

impl HuaweicloudProvider {
    // ==================== 辅助方法 ====================

    /// 统一处理华为云响应错误
    fn handle_response_error(
        &self,
        status: u16,
        response_text: &str,
        ctx: ErrorContext,
    ) -> Result<()> {
        if (200..300).contains(&status) {
            return Ok(());
        }

        // 尝试解析结构化错误
        if let Ok(error) = serde_json::from_str::<ErrorResponse>(response_text) {
            return Err(self.map_error(
                RawApiError::with_code(
                    error.code.unwrap_or_default(),
                    error.message.unwrap_or_default(),
                ),
                ctx,
            ));
        }

        // 回退到通用错误
        Err(self.unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}"))))
    }

    /// 执行带 body 的请求（POST/PUT）
    async fn request_with_body<T, B>(
        &self,
        method: &str,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize,
    {
        let payload =
            serde_json::to_string(body).map_err(|e| ProviderError::SerializationError {
                provider: self.provider_name().to_string(),
                detail: e.to_string(),
            })?;

        log::debug!("Request Body: {payload}");

        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let authorization = self.sign(method, path, "", &headers, &payload, &timestamp);
        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");

        // 根据 method 构建请求
        let request_builder = match method {
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            _ => unreachable!("request_with_body only supports POST/PUT"),
        };

        let request = request_builder
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .body(payload);

        let (status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            method,
            &url,
            self.max_retries,
        )
        .await?;

        self.handle_response_error(status, &response_text, ctx)?;
        HttpUtils::parse_json(&response_text, self.provider_name())
    }

    // ==================== 公开 API 方法 ====================

    /// 执行 GET 请求
    pub(crate) async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        query: &str,
        ctx: ErrorContext,
    ) -> Result<T> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("GET", path, query, &headers, "", &timestamp);

        let url = if query.is_empty() {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}")
        } else {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}?{query}")
        };

        let request = self
            .client
            .get(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization);

        let (status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "GET",
            &url,
            self.max_retries,
        )
        .await?;

        self.handle_response_error(status, &response_text, ctx)?;
        HttpUtils::parse_json(&response_text, self.provider_name())
    }

    /// 执行 POST 请求
    pub(crate) async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        self.request_with_body("POST", path, body, ctx).await
    }

    /// 执行 PUT 请求
    pub(crate) async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        self.request_with_body("PUT", path, body, ctx).await
    }

    /// 执行 DELETE 请求
    pub(crate) async fn delete(&self, path: &str, ctx: ErrorContext) -> Result<()> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("DELETE", path, "", &headers, "", &timestamp);
        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");

        let request = self
            .client
            .delete(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization);

        let (status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "DELETE",
            &url,
            self.max_retries,
        )
        .await?;

        self.handle_response_error(status, &response_text, ctx)
    }
}
