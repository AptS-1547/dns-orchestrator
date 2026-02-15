//! Account import/export service.

use std::collections::HashSet;
use std::sync::Arc;

use dns_orchestrator_provider::ProviderCredentials;

use crate::crypto;
use crate::error::{CoreError, CoreResult};
use crate::services::account_service::AccountService;
use crate::types::{
    Account, ExportAccountsRequest, ExportAccountsResponse, ExportFile, ExportFileHeader,
    ExportedAccount, ImportAccountsRequest, ImportFailure, ImportPreview, ImportPreviewAccount,
    ImportResult,
};

/// Service that handles account export, preview, and import.
pub struct ImportExportService {
    account_service: Arc<AccountService>,
}

impl ImportExportService {
    /// Creates an import/export service.
    #[must_use]
    pub fn new(account_service: Arc<AccountService>) -> Self {
        Self { account_service }
    }

    /// Parses an export file and optionally decrypts account payloads.
    ///
    /// Returns the parsed file plus:
    /// - `Some(accounts)` when payload data is available (plaintext/decrypted)
    /// - `None` when the file is encrypted and no password is provided
    fn parse_and_decrypt_accounts(
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<(ExportFile, Option<Vec<ExportedAccount>>)> {
        // 1. Parse file content.
        let export_file: ExportFile = serde_json::from_str(content)
            .map_err(|e| CoreError::ImportExportError(format!("Invalid import file: {e}")))?;

        // 2. Resolve KDF parameters from file version (encrypted files only).
        let kdf_iterations = if export_file.header.encrypted {
            crypto::get_pbkdf2_iterations(export_file.header.version).ok_or_else(|| {
                CoreError::ImportExportError(format!(
                    "Unsupported file version: {}",
                    export_file.header.version
                ))
            })?
        } else {
            0 // Not used for plaintext files.
        };

        // 3. If encrypted without a password, report "password required" via `None`.
        if export_file.header.encrypted && password.is_none() {
            return Ok((export_file, None));
        }

        // 4. Decrypt payload or parse plaintext JSON.
        let accounts: Vec<ExportedAccount> = if export_file.header.encrypted {
            let password = password.ok_or_else(|| {
                CoreError::ImportExportError("Password is required for encrypted files".to_string())
            })?;

            log::info!(
                "Decrypting version {} file using PBKDF2-HMAC-SHA256 ({} iterations)",
                export_file.header.version,
                kdf_iterations
            );

            let ciphertext = export_file.data.as_str().ok_or_else(|| {
                CoreError::ImportExportError("Invalid encrypted payload".to_string())
            })?;
            let salt = export_file.header.salt.as_ref().ok_or_else(|| {
                CoreError::ImportExportError("Missing encryption salt".to_string())
            })?;
            let nonce = export_file.header.nonce.as_ref().ok_or_else(|| {
                CoreError::ImportExportError("Missing encryption nonce".to_string())
            })?;

            // Decrypt using the iteration count mapped to this file version.
            let plaintext =
                crypto::decrypt_with_iterations(ciphertext, password, salt, nonce, kdf_iterations)
                    .map_err(|_| {
                        CoreError::ImportExportError(
                            "Decryption failed. Please check whether the password is correct"
                                .to_string(),
                        )
                    })?;

            serde_json::from_slice(&plaintext).map_err(|e| {
                CoreError::ImportExportError(format!("Failed to parse account data: {e}"))
            })?
        } else {
            serde_json::from_value(export_file.data.clone()).map_err(|e| {
                CoreError::ImportExportError(format!("Failed to parse account data: {e}"))
            })?
        };

        Ok((export_file, Some(accounts)))
    }

    /// Exports selected accounts to `.dnso` JSON content.
    ///
    /// # Arguments
    /// * `request` - Export request including account IDs and encryption options.
    /// * `app_version` - Current application version.
    pub async fn export_accounts(
        &self,
        request: ExportAccountsRequest,
        app_version: &str,
    ) -> CoreResult<ExportAccountsResponse> {
        // 1. Load metadata for selected accounts.
        let all_accounts = self.account_service.list_accounts().await?;
        let selected_accounts: Vec<&Account> = all_accounts
            .iter()
            .filter(|a| request.account_ids.contains(&a.id))
            .collect();

        if selected_accounts.is_empty() {
            return Err(CoreError::NoAccountsSelected);
        }

        // 2. Load credentials and build export entries.
        let mut exported_accounts = Vec::new();
        for account in selected_accounts {
            let Ok(credentials) = self.account_service.load_credentials(&account.id).await else {
                log::warn!(
                    "Failed to load credentials for account {}, skipping export",
                    account.id
                );
                continue;
            };

            // Convert structured credentials to a serializable map.
            let credentials_map = credentials.to_map();

            exported_accounts.push(ExportedAccount {
                // Use fresh IDs to avoid collisions during import.
                id: uuid::Uuid::new_v4().to_string(),
                name: account.name.clone(),
                provider: account.provider.clone(),
                created_at: account.created_at,
                updated_at: account.updated_at,
                credentials: credentials_map,
            });
        }

        // 3. Serialize account data.
        let accounts_json = serde_json::to_value(&exported_accounts)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        // 4. Build export file structure.
        let now = chrono::Utc::now();

        let export_file = if request.encrypt {
            let password = request.password.as_ref().ok_or_else(|| {
                CoreError::ValidationError("Password is required for encrypted export".to_string())
            })?;

            let plaintext = serde_json::to_vec(&accounts_json)
                .map_err(|e| CoreError::SerializationError(e.to_string()))?;

            let (salt, nonce, ciphertext) = crypto::encrypt(&plaintext, password)
                .map_err(|e| CoreError::ImportExportError(e.to_string()))?;

            ExportFile {
                header: ExportFileHeader {
                    version: crypto::CURRENT_FILE_VERSION,
                    encrypted: true,
                    salt: Some(salt),
                    nonce: Some(nonce),
                    exported_at: now.to_rfc3339(),
                    app_version: app_version.to_string(),
                },
                data: serde_json::Value::String(ciphertext),
            }
        } else {
            ExportFile {
                header: ExportFileHeader {
                    version: crypto::CURRENT_FILE_VERSION,
                    encrypted: false,
                    salt: None,
                    nonce: None,
                    exported_at: now.to_rfc3339(),
                    app_version: app_version.to_string(),
                },
                data: accounts_json,
            }
        };

        // 5. Render pretty JSON output.
        let content = serde_json::to_string_pretty(&export_file)
            .map_err(|e| CoreError::SerializationError(e.to_string()))?;

        let suggested_filename = format!(
            "dns-orchestrator-backup-{}.dnso",
            chrono::Local::now().format("%Y%m%d-%H%M%S")
        );

        Ok(ExportAccountsResponse {
            content,
            suggested_filename,
        })
    }

    /// Previews an import file without applying any writes.
    pub async fn preview_import(
        &self,
        content: &str,
        password: Option<&str>,
    ) -> CoreResult<ImportPreview> {
        // 1. Parse/decrypt content.
        let (export_file, accounts_opt) = Self::parse_and_decrypt_accounts(content, password)?;

        // 2. If password is required but missing, return an encrypted preview state.
        let Some(accounts) = accounts_opt else {
            return Ok(ImportPreview {
                encrypted: true,
                account_count: 0,
                accounts: None,
            });
        };

        // 3. Mark name conflicts with existing accounts.
        let existing_accounts = self.account_service.list_accounts().await?;
        let existing_names: HashSet<_> =
            existing_accounts.iter().map(|a| a.name.as_str()).collect();

        let preview_accounts: Vec<ImportPreviewAccount> = accounts
            .iter()
            .map(|a| ImportPreviewAccount {
                name: a.name.clone(),
                provider: a.provider.clone(),
                has_conflict: existing_names.contains(a.name.as_str()),
            })
            .collect();

        Ok(ImportPreview {
            encrypted: export_file.header.encrypted,
            account_count: accounts.len(),
            accounts: Some(preview_accounts),
        })
    }

    /// Imports accounts from file content.
    pub async fn import_accounts(
        &self,
        request: ImportAccountsRequest,
    ) -> CoreResult<ImportResult> {
        // 1. Parse/decrypt input.
        let (_, accounts_opt) =
            Self::parse_and_decrypt_accounts(&request.content, request.password.as_deref())?;

        let accounts = accounts_opt.ok_or_else(|| {
            CoreError::ImportExportError("Password is required for encrypted files".to_string())
        })?;

        // 2. Import accounts one by one and collect failures.
        let mut success_count = 0;
        let mut failures = Vec::new();

        for exported in accounts {
            let credentials =
                match ProviderCredentials::from_map(&exported.provider, &exported.credentials) {
                    Ok(c) => c,
                    Err(e) => {
                        failures.push(ImportFailure {
                            name: exported.name.clone(),
                            reason: format!("Invalid credential format: {e}"),
                        });
                        continue;
                    }
                };

            match self
                .account_service
                .create_account_from_import(exported.name.clone(), exported.provider, credentials)
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => failures.push(ImportFailure {
                    name: exported.name,
                    reason: e.to_string(),
                }),
            }
        }

        Ok(ImportResult {
            success_count,
            failures,
        })
    }
}
