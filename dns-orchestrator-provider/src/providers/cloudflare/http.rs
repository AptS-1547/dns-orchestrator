//! Cloudflare HTTP 请求方法（重构版：使用通用 HTTP 工具）

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::http_client::HttpUtils;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};
use crate::types::PaginationParams;

use super::{
    CF_API_BASE, CloudflareDnsRecord, CloudflareProvider, CloudflareResponse, MAX_PAGE_SIZE_ZONES,
};

impl CloudflareProvider {
    /// 执行 GET 请求
    pub(crate) async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        ctx: ErrorContext,
    ) -> Result<T> {
        let url = format!("{CF_API_BASE}{path}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token));

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "GET",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<T> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段"))
    }

    /// 执行 GET 请求 (带分页)
    pub(crate) async fn get_paginated<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        params: &PaginationParams,
        ctx: ErrorContext,
    ) -> Result<(Vec<T>, u32)> {
        // Cloudflare zones API 最大 per_page 是 50
        let url = format!(
            "{}{}?page={}&per_page={}",
            CF_API_BASE,
            path,
            params.page,
            params.page_size.min(MAX_PAGE_SIZE_ZONES)
        );

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token));

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "GET",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<Vec<T>> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        let total_count = cf_response.result_info.map_or(0, |i| i.total_count);
        let items = cf_response.result.unwrap_or_default();

        Ok((items, total_count))
    }

    /// 执行 GET 请求 (带自定义 URL，用于 list_records)
    pub(crate) async fn get_records(
        &self,
        url: &str,
        ctx: ErrorContext,
    ) -> Result<(Vec<CloudflareDnsRecord>, u32)> {
        let full_url = format!("{CF_API_BASE}{url}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .get(&full_url)
            .header("Authorization", format!("Bearer {}", self.api_token));

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "GET",
            &full_url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<Vec<CloudflareDnsRecord>> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        let total_count = cf_response.result_info.map_or(0, |i| i.total_count);
        let records = cf_response.result.unwrap_or_default();

        Ok((records, total_count))
    }

    /// 执行 POST 请求
    pub(crate) async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        let url = format!("{CF_API_BASE}{path}");
        let body_json =
            serde_json::to_string_pretty(body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("Request Body: {body_json}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(body);

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "POST",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<T> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段"))
    }

    /// 执行 PATCH 请求
    pub(crate) async fn patch<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        let url = format!("{CF_API_BASE}{path}");
        let body_json =
            serde_json::to_string_pretty(body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("Request Body: {body_json}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(body);

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "PATCH",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<T> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段"))
    }

    /// 执行 POST 请求（直接使用 JSON Value）
    pub(crate) async fn post_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: serde_json::Value,
        ctx: ErrorContext,
    ) -> Result<T> {
        let url = format!("{CF_API_BASE}{path}");
        let body_json =
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("Request Body: {body_json}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body);

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "POST",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<T> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段"))
    }

    /// 执行 PATCH 请求（直接使用 JSON Value）
    pub(crate) async fn patch_json<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        body: serde_json::Value,
        ctx: ErrorContext,
    ) -> Result<T> {
        let url = format!("{CF_API_BASE}{path}");
        let body_json =
            serde_json::to_string_pretty(&body).unwrap_or_else(|_| "无法序列化请求体".to_string());
        log::debug!("Request Body: {body_json}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body);

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "PATCH",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<T> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        cf_response
            .result
            .ok_or_else(|| self.parse_error("响应中缺少 result 字段"))
    }

    /// 执行 DELETE 请求
    pub(crate) async fn delete(&self, path: &str, ctx: ErrorContext) -> Result<()> {
        let url = format!("{CF_API_BASE}{path}");

        // 使用 HttpUtils 发送请求（带重试）
        let request = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token));

        let (_status, response_text) = HttpUtils::execute_request_with_retry(
            request,
            self.provider_name(),
            "DELETE",
            &url,
            self.max_retries,
        )
        .await?;

        // 解析 Cloudflare 响应
        let cf_response: CloudflareResponse<serde_json::Value> =
            HttpUtils::parse_json(&response_text, self.provider_name())?;

        // 处理 API 错误
        if !cf_response.success {
            let (code, message) = cf_response
                .errors
                .and_then(|errors| {
                    errors
                        .first()
                        .map(|e| (e.code.to_string(), e.message.clone()))
                })
                .unwrap_or_else(|| (String::new(), "Unknown error".to_string()));
            log::error!("API 错误: {message}");
            return Err(self.map_error(RawApiError::with_code(code, message), ctx));
        }

        Ok(())
    }
}
