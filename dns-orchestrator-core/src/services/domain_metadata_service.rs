//! Domain metadata management service.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::CoreResult;
use crate::traits::DomainMetadataRepository;
use crate::types::{
    BatchTagFailure, BatchTagRequest, BatchTagResult, DomainMetadata, DomainMetadataKey,
    DomainMetadataUpdate,
};

/// Service for reading and mutating domain metadata.
pub struct DomainMetadataService {
    repository: Arc<dyn DomainMetadataRepository>,
}

impl DomainMetadataService {
    /// Creates a metadata service.
    #[must_use]
    pub fn new(repository: Arc<dyn DomainMetadataRepository>) -> Self {
        Self { repository }
    }

    /// Gets metadata for one domain.
    ///
    /// Returns default metadata when no stored entry exists.
    pub async fn get_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
    ) -> CoreResult<DomainMetadata> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok(self.repository.find_by_key(&key).await?.unwrap_or_default())
    }

    /// Gets metadata in batch (used by domain list APIs).
    pub async fn get_metadata_batch(
        &self,
        keys: Vec<(String, String)>, // `(account_id, domain_id)` pairs.
    ) -> CoreResult<HashMap<DomainMetadataKey, DomainMetadata>> {
        let keys: Vec<DomainMetadataKey> = keys
            .into_iter()
            .map(|(acc, dom)| DomainMetadataKey::new(acc, dom))
            .collect();
        self.repository.find_by_keys(&keys).await
    }

    /// Saves full metadata for one domain.
    pub async fn save_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
        metadata: DomainMetadata,
    ) -> CoreResult<()> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.save(&key, &metadata).await
    }

    /// Applies a partial metadata update.
    pub async fn update_metadata(
        &self,
        account_id: &str,
        domain_id: &str,
        update: DomainMetadataUpdate,
    ) -> CoreResult<()> {
        use crate::error::CoreError;

        // Validate color (`none` means no color).
        const VALID_COLORS: &[&str] = &[
            "red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "brown", "gray",
            "none",
        ];

        if let Some(ref color) = update.color
            && !VALID_COLORS.contains(&color.as_str())
        {
            return Err(CoreError::ValidationError(format!(
                "Invalid color key: '{}'. Must be one of: {}",
                color,
                VALID_COLORS.join(", ")
            )));
        }

        // Validate note length (only when note is provided).
        if let Some(Some(ref note)) = update.note
            && note.len() > 500
        {
            return Err(CoreError::ValidationError(
                "Note length cannot exceed 500 characters".to_string(),
            ));
        }

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.update(&key, &update).await
    }

    /// Deletes metadata for one domain.
    pub async fn delete_metadata(&self, account_id: &str, domain_id: &str) -> CoreResult<()> {
        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        self.repository.delete(&key).await
    }

    /// Toggles favorite state.
    pub async fn toggle_favorite(&self, account_id: &str, domain_id: &str) -> CoreResult<bool> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;
        metadata.is_favorite = !metadata.is_favorite;

        // Record favorite timestamp once on the first favorite action.
        if metadata.is_favorite && metadata.favorited_at.is_none() {
            metadata.favorited_at = Some(chrono::Utc::now());
        }
        // Note: `favorited_at` is intentionally not cleared when unfavoriting.

        metadata.touch();

        let new_state = metadata.is_favorite;
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(new_state)
    }

    /// Lists favorite domain keys under one account.
    pub async fn list_favorites(&self, account_id: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        self.repository.find_favorites_by_account(account_id).await
    }

    /// Deletes all metadata under one account.
    pub async fn delete_account_metadata(&self, account_id: &str) -> CoreResult<()> {
        self.repository.delete_by_account(account_id).await
    }

    /// Validates one tag.
    ///
    /// # Validation rules
    /// - Must not be empty after trimming.
    /// - Max length is 50 characters.
    fn validate_tag(tag: &str) -> CoreResult<()> {
        use crate::error::CoreError;

        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Err(CoreError::ValidationError(
                "Tag cannot be empty".to_string(),
            ));
        }
        if trimmed.len() > 50 {
            return Err(CoreError::ValidationError(
                "Tag length cannot exceed 50 characters".to_string(),
            ));
        }
        Ok(())
    }

    /// Adds one tag and returns updated tags.
    pub async fn add_tag(
        &self,
        account_id: &str,
        domain_id: &str,
        tag: String,
    ) -> CoreResult<Vec<String>> {
        use crate::error::CoreError;

        // Validate incoming tag.
        let tag = tag.trim().to_string();
        Self::validate_tag(&tag)?;

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Deduplicate: no-op if already exists.
        if metadata.tags.contains(&tag) {
            return Ok(metadata.tags);
        }

        // Enforce tag count limit.
        if metadata.tags.len() >= 10 {
            return Err(CoreError::ValidationError(
                "Cannot add more than 10 tags".to_string(),
            ));
        }

        metadata.tags.push(tag);
        metadata.tags.sort();
        metadata.touch();

        let tags = metadata.tags.clone();
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(tags)
    }

    /// Removes one tag and returns updated tags.
    pub async fn remove_tag(
        &self,
        account_id: &str,
        domain_id: &str,
        tag: &str,
    ) -> CoreResult<Vec<String>> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Silent no-op when the tag does not exist.
        metadata.tags.retain(|t| t != tag);
        metadata.touch();

        let tags = metadata.tags.clone();
        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(tags)
    }

    /// Replaces all tags and returns updated tags.
    pub async fn set_tags(
        &self,
        account_id: &str,
        domain_id: &str,
        tags: Vec<String>,
    ) -> CoreResult<Vec<String>> {
        use crate::error::CoreError;

        // Validate each tag.
        for tag in &tags {
            Self::validate_tag(tag)?;
        }

        if tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot have more than 10 tags".to_string(),
            ));
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Normalize, deduplicate, and sort.
        let mut cleaned_tags: Vec<String> = tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        cleaned_tags.sort();
        cleaned_tags.dedup();

        metadata.tags = cleaned_tags.clone();
        metadata.touch();

        self.save_metadata(account_id, domain_id, metadata).await?;
        Ok(cleaned_tags)
    }

    /// Finds domains by tag (cross-account).
    pub async fn find_by_tag(&self, tag: &str) -> CoreResult<Vec<DomainMetadataKey>> {
        self.repository.find_by_tag(tag).await
    }

    /// Lists all tags currently in use.
    pub async fn list_all_tags(&self) -> CoreResult<Vec<String>> {
        self.repository.list_all_tags().await
    }

    // ===== Batch tag operations =====

    /// Batch add tags to domains.
    pub async fn batch_add_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // Phase 1: process all updates in memory.
        for req in requests {
            match self
                .add_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // Phase 2: persist all successful updates in one batch call.
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    /// Batch remove tags from domains.
    pub async fn batch_remove_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // Phase 1: process all updates in memory.
        for req in requests {
            match self
                .remove_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // Phase 2: persist all successful updates in one batch call.
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    /// Batch replace tags for domains.
    pub async fn batch_set_tags(
        &self,
        requests: Vec<BatchTagRequest>,
    ) -> CoreResult<BatchTagResult> {
        let mut entries_to_save = Vec::new();
        let mut failures = Vec::new();

        // Phase 1: process all updates in memory.
        for req in requests {
            match self
                .set_tags_internal_no_save(&req.account_id, &req.domain_id, req.tags)
                .await
            {
                Ok((key, metadata)) => entries_to_save.push((key, metadata)),
                Err(e) => failures.push(BatchTagFailure {
                    account_id: req.account_id,
                    domain_id: req.domain_id,
                    reason: e.to_string(),
                }),
            }
        }

        // Phase 2: persist all successful updates in one batch call.
        if !entries_to_save.is_empty() {
            self.repository.batch_save(&entries_to_save).await?;
        }

        Ok(BatchTagResult {
            success_count: entries_to_save.len(),
            failed_count: failures.len(),
            failures,
        })
    }

    // ===== Internal helpers for batch optimizations =====

    /// Adds tags for one domain in-memory (no persistence).
    async fn add_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags_to_add: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        use crate::error::CoreError;

        // Validate each tag.
        for tag in &tags_to_add {
            Self::validate_tag(tag)?;
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Merge existing and incoming tags.
        let mut all_tags: Vec<String> = metadata.tags.clone();
        for tag in tags_to_add {
            let trimmed = tag.trim().to_string();
            if !trimmed.is_empty() && !all_tags.contains(&trimmed) {
                all_tags.push(trimmed);
            }
        }

        // Enforce tag count limit.
        if all_tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot exceed 10 tags".to_string(),
            ));
        }

        all_tags.sort();
        all_tags.dedup();

        metadata.tags = all_tags;
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }

    /// Removes tags for one domain in-memory (no persistence).
    async fn remove_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags_to_remove: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Remove specified tags.
        let tags_to_remove_set: std::collections::HashSet<String> = tags_to_remove
            .into_iter()
            .map(|t| t.trim().to_string())
            .collect();

        metadata.tags.retain(|t| !tags_to_remove_set.contains(t));
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }

    /// Replaces tags for one domain in-memory (no persistence).
    async fn set_tags_internal_no_save(
        &self,
        account_id: &str,
        domain_id: &str,
        tags: Vec<String>,
    ) -> CoreResult<(DomainMetadataKey, DomainMetadata)> {
        use crate::error::CoreError;

        // Validate each tag.
        for tag in &tags {
            Self::validate_tag(tag)?;
        }

        if tags.len() > 10 {
            return Err(CoreError::ValidationError(
                "Cannot have more than 10 tags".to_string(),
            ));
        }

        let mut metadata = self.get_metadata(account_id, domain_id).await?;

        // Normalize, deduplicate, and sort.
        let mut cleaned_tags: Vec<String> = tags
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        cleaned_tags.sort();
        cleaned_tags.dedup();

        metadata.tags = cleaned_tags;
        metadata.touch();

        let key = DomainMetadataKey::new(account_id.to_string(), domain_id.to_string());
        Ok((key, metadata))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::MockDomainMetadataRepository;

    fn make_service() -> DomainMetadataService {
        DomainMetadataService::new(Arc::new(MockDomainMetadataRepository::new()))
    }

    #[tokio::test]
    async fn get_metadata_returns_default_when_not_found() {
        let svc = make_service();
        let m = svc.get_metadata("acc", "dom").await.unwrap();
        assert!(m.is_empty());
        assert!(!m.is_favorite);
    }

    #[tokio::test]
    async fn toggle_favorite_on_off() {
        let svc = make_service();

        // First toggle -> on.
        let state = svc.toggle_favorite("a", "d").await.unwrap();
        assert!(state);

        let m = svc.get_metadata("a", "d").await.unwrap();
        assert!(m.is_favorite);

        // Second toggle -> off.
        let state = svc.toggle_favorite("a", "d").await.unwrap();
        assert!(!state);

        let m = svc.get_metadata("a", "d").await.unwrap();
        assert!(!m.is_favorite);
    }

    #[tokio::test]
    async fn toggle_favorite_records_favorited_at_once() {
        let svc = make_service();

        // Favorite -> set `favorited_at`.
        svc.toggle_favorite("a", "d").await.unwrap();
        let m = svc.get_metadata("a", "d").await.unwrap();
        assert!(m.favorited_at.is_some());
        let first_fav_at = m.favorited_at.unwrap();

        // Unfavorite -> `favorited_at` unchanged.
        svc.toggle_favorite("a", "d").await.unwrap();
        let m = svc.get_metadata("a", "d").await.unwrap();
        assert_eq!(m.favorited_at, Some(first_fav_at));

        // Favorite again -> `favorited_at` remains unchanged.
        svc.toggle_favorite("a", "d").await.unwrap();
        let m = svc.get_metadata("a", "d").await.unwrap();
        assert_eq!(m.favorited_at, Some(first_fav_at));
    }

    #[tokio::test]
    async fn add_tag_success() {
        let svc = make_service();
        let tags = svc.add_tag("a", "d", "web".to_string()).await.unwrap();
        assert_eq!(tags, vec!["web"]);
    }

    #[tokio::test]
    async fn add_tag_duplicate_ignored() {
        let svc = make_service();
        svc.add_tag("a", "d", "web".to_string()).await.unwrap();
        let tags = svc.add_tag("a", "d", "web".to_string()).await.unwrap();
        assert_eq!(tags, vec!["web"]);
    }

    #[tokio::test]
    async fn add_tag_exceeds_limit() {
        let svc = make_service();
        for i in 0..10 {
            svc.add_tag("a", "d", format!("tag{i}")).await.unwrap();
        }
        let result = svc.add_tag("a", "d", "overflow".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_tag_empty_rejected() {
        let svc = make_service();
        let result = svc.add_tag("a", "d", "  ".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn add_tag_too_long_rejected() {
        let svc = make_service();
        let long_tag = "a".repeat(51);
        let result = svc.add_tag("a", "d", long_tag).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn remove_tag_success() {
        let svc = make_service();
        svc.add_tag("a", "d", "web".to_string()).await.unwrap();
        let tags = svc.remove_tag("a", "d", "web").await.unwrap();
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn remove_tag_nonexistent_silent() {
        let svc = make_service();
        // Removing non-existing tags should not fail.
        let tags = svc.remove_tag("a", "d", "ghost").await.unwrap();
        assert!(tags.is_empty());
    }

    #[tokio::test]
    async fn set_tags_dedup_and_sort() {
        let svc = make_service();
        let tags = svc
            .set_tags("a", "d", vec!["b".into(), "a".into(), "b".into()])
            .await
            .unwrap();
        assert_eq!(tags, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn set_tags_exceeds_limit() {
        let svc = make_service();
        let tags: Vec<String> = (0..11).map(|i| format!("tag{i}")).collect();
        let result = svc.set_tags("a", "d", tags).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn validate_color_valid() {
        let svc = make_service();
        let update = DomainMetadataUpdate {
            is_favorite: None,
            tags: None,
            color: Some("red".to_string()),
            note: None,
        };
        let result = svc.update_metadata("a", "d", update).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_color_invalid() {
        let svc = make_service();
        let update = DomainMetadataUpdate {
            is_favorite: None,
            tags: None,
            color: Some("rainbow".to_string()),
            note: None,
        };
        let result = svc.update_metadata("a", "d", update).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn validate_note_too_long() {
        let svc = make_service();
        let update = DomainMetadataUpdate {
            is_favorite: None,
            tags: None,
            color: None,
            note: Some(Some("x".repeat(501))),
        };
        let result = svc.update_metadata("a", "d", update).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn batch_add_tags() {
        let svc = make_service();
        let requests = vec![
            BatchTagRequest {
                account_id: "a".into(),
                domain_id: "d1".into(),
                tags: vec!["shared".into()],
            },
            BatchTagRequest {
                account_id: "a".into(),
                domain_id: "d2".into(),
                tags: vec!["shared".into()],
            },
        ];
        let result = svc.batch_add_tags(requests).await.unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 0);

        let m1 = svc.get_metadata("a", "d1").await.unwrap();
        let m2 = svc.get_metadata("a", "d2").await.unwrap();
        assert_eq!(m1.tags, vec!["shared"]);
        assert_eq!(m2.tags, vec!["shared"]);
    }

    #[tokio::test]
    async fn batch_remove_tags() {
        let svc = make_service();

        // Seed initial tags.
        svc.add_tag("a", "d1", "web".to_string()).await.unwrap();
        svc.add_tag("a", "d2", "web".to_string()).await.unwrap();

        let requests = vec![
            BatchTagRequest {
                account_id: "a".into(),
                domain_id: "d1".into(),
                tags: vec!["web".into()],
            },
            BatchTagRequest {
                account_id: "a".into(),
                domain_id: "d2".into(),
                tags: vec!["web".into()],
            },
        ];
        let result = svc.batch_remove_tags(requests).await.unwrap();
        assert_eq!(result.success_count, 2);
        assert_eq!(result.failed_count, 0);

        let m1 = svc.get_metadata("a", "d1").await.unwrap();
        let m2 = svc.get_metadata("a", "d2").await.unwrap();
        assert!(m1.tags.is_empty());
        assert!(m2.tags.is_empty());
    }
}
