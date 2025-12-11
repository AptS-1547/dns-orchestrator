//! RPC 风格的统一调用入口
//!
//! 所有前端请求都通过 POST /api/invoke 进行

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ApiError, ApiResponse};
use crate::state::AppState;

use super::{account, dns, domain, toolbox};

/// RPC 请求
#[derive(Debug, Deserialize)]
pub struct InvokeRequest {
    pub command: String,
    #[serde(default)]
    pub args: Value,
}

/// RPC 响应
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum InvokeResponse {
    Success(Value),
    Error(ApiResponse<()>),
}

/// 统一调用入口
pub async fn invoke_handler(
    state: web::Data<AppState>,
    req: web::Json<InvokeRequest>,
) -> HttpResponse {
    tracing::debug!("收到 RPC 请求: {} {:?}", req.command, req.args);

    let result = dispatch_command(&state, &req.command, req.args.clone()).await;

    match result {
        Ok(value) => HttpResponse::Ok().json(value),
        Err(e) => {
            tracing::error!("命令 {} 执行失败: {}", req.command, e);
            e.error_response()
        }
    }
}

/// 命令分发
async fn dispatch_command(
    state: &AppState,
    command: &str,
    args: Value,
) -> Result<Value, ApiError> {
    match command {
        // Account commands
        "list_accounts" => {
            let result = account::list_accounts(state).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "create_account" => {
            let req: account::CreateAccountArgs = serde_json::from_value(args)?;
            let result = account::create_account(state, req.request).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "delete_account" => {
            let req: account::DeleteAccountArgs = serde_json::from_value(args)?;
            account::delete_account(state, &req.account_id).await?;
            Ok(serde_json::to_value(ApiResponse::success(()))?)
        }
        "list_providers" => {
            let result = account::list_providers();
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "export_accounts" => {
            let req: account::ExportAccountsArgs = serde_json::from_value(args)?;
            let result = account::export_accounts(state, req.request).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "preview_import" => {
            let req: account::PreviewImportArgs = serde_json::from_value(args)?;
            let result = account::preview_import(&req.content, req.password.as_deref())?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "import_accounts" => {
            let req: account::ImportAccountsArgs = serde_json::from_value(args)?;
            let result = account::import_accounts(state, req.request).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }

        // Domain commands
        "list_domains" => {
            let req: domain::ListDomainsArgs = serde_json::from_value(args)?;
            let result = domain::list_domains(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "get_domain" => {
            let req: domain::GetDomainArgs = serde_json::from_value(args)?;
            let result = domain::get_domain(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }

        // DNS commands
        "list_dns_records" => {
            let req: dns::ListDnsRecordsArgs = serde_json::from_value(args)?;
            let result = dns::list_dns_records(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "create_dns_record" => {
            let req: dns::CreateDnsRecordArgs = serde_json::from_value(args)?;
            let result = dns::create_dns_record(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "update_dns_record" => {
            let req: dns::UpdateDnsRecordArgs = serde_json::from_value(args)?;
            let result = dns::update_dns_record(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "delete_dns_record" => {
            let req: dns::DeleteDnsRecordArgs = serde_json::from_value(args)?;
            dns::delete_dns_record(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(()))?)
        }
        "batch_delete_dns_records" => {
            let req: dns::BatchDeleteDnsRecordsArgs = serde_json::from_value(args)?;
            let result = dns::batch_delete_dns_records(state, req).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }

        // Toolbox commands
        "whois_lookup" => {
            let req: toolbox::WhoisLookupArgs = serde_json::from_value(args)?;
            let result = toolbox::whois_lookup(&req.domain).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "dns_lookup" => {
            let req: toolbox::DnsLookupArgs = serde_json::from_value(args)?;
            let result =
                toolbox::dns_lookup(&req.domain, &req.record_type, req.nameserver.as_deref())
                    .await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "ip_lookup" => {
            let req: toolbox::IpLookupArgs = serde_json::from_value(args)?;
            let result = toolbox::ip_lookup(&req.query).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }
        "ssl_check" => {
            let req: toolbox::SslCheckArgs = serde_json::from_value(args)?;
            let result = toolbox::ssl_check(&req.domain, req.port).await?;
            Ok(serde_json::to_value(ApiResponse::success(result))?)
        }

        // 不支持的命令
        _ => Err(ApiError::UnknownCommand(command.to_string())),
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::BadRequest(format!("JSON 解析错误: {err}"))
    }
}
