use dns_orchestrator_core::error::CoreError;
use serde::Serialize;

/// 应用层错误 — 直接包装 `CoreError`
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct AppError(pub CoreError);

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<CoreError> for AppError {
    fn from(err: CoreError) -> Self {
        if err.is_expected() {
            log::warn!("AppError: {err}");
        } else {
            log::error!("AppError: {err}");
        }
        Self(err)
    }
}

impl From<dns_orchestrator_toolbox::ToolboxError> for AppError {
    fn from(err: dns_orchestrator_toolbox::ToolboxError) -> Self {
        use dns_orchestrator_toolbox::ToolboxError;
        let core_err = match err {
            ToolboxError::ValidationError(msg) => {
                let e = CoreError::ValidationError(msg);
                log::warn!("ToolboxError: {e}");
                e
            }
            ToolboxError::NetworkError(msg) => {
                let e = CoreError::NetworkError(msg);
                log::error!("ToolboxError: {e}");
                e
            }
        };
        Self(core_err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dns_orchestrator_toolbox::ToolboxError;

    #[test]
    fn test_from_core_error() {
        let core_err = CoreError::AccountNotFound("test-id".into());
        let app_err = AppError::from(core_err);
        assert!(matches!(app_err.0, CoreError::AccountNotFound(_)));
    }

    #[test]
    fn test_display_matches_inner() {
        let core_err = CoreError::ValidationError("bad input".into());
        let expected = core_err.to_string();
        let app_err = AppError::from(core_err);
        assert_eq!(app_err.to_string(), expected);
    }

    #[test]
    fn test_from_toolbox_validation_error() {
        let err = ToolboxError::ValidationError("invalid domain".into());
        let app_err = AppError::from(err);
        match &app_err.0 {
            CoreError::ValidationError(msg) => assert_eq!(msg, "invalid domain"),
            other => panic!("expected ValidationError, got {other:?}"),
        }
    }

    #[test]
    fn test_from_toolbox_network_error() {
        let err = ToolboxError::NetworkError("timeout".into());
        let app_err = AppError::from(err);
        match &app_err.0 {
            CoreError::NetworkError(msg) => assert_eq!(msg, "timeout"),
            other => panic!("expected NetworkError, got {other:?}"),
        }
    }
}
