//! Domain metadata types.

use serde::{Deserialize, Serialize};

/// Default color marker used when no color is assigned.
fn default_color() -> String {
    "none".to_string()
}

/// Domain metadata key (composite key).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataKey {
    /// Account ID.
    pub account_id: String,
    /// Domain ID.
    pub domain_id: String,
}

impl DomainMetadataKey {
    /// Creates a new metadata key.
    #[must_use]
    pub fn new(account_id: String, domain_id: String) -> Self {
        Self {
            account_id,
            domain_id,
        }
    }

    /// Converts this key into a storage key (`account_id::domain_id`).
    #[must_use]
    pub fn to_storage_key(&self) -> String {
        format!("{}::{}", self.account_id, self.domain_id)
    }

    /// Parses from a storage key.
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

/// User-defined domain metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadata {
    /// Whether the domain is favorited.
    #[serde(default)]
    pub is_favorite: bool,

    /// User tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Color label (`none` means no color).
    #[serde(default = "default_color")]
    pub color: String,

    /// Optional note.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Timestamp of first time being favorited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favorited_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last updated timestamp.
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
    /// Creates metadata with all fields explicitly provided.
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

    /// Updates `updated_at` to current time.
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now();
    }

    /// Returns `true` when metadata is effectively empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !self.is_favorite
            && self.tags.is_empty()
            && self.color == "none"
            && self.note.is_none()
            && self.favorited_at.is_none()
    }
}

/// Partial update payload for domain metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainMetadataUpdate {
    /// New favorite state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_favorite: Option<bool>,

    /// New full tag set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// New color value. Empty string means "clear color".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Optional note patch:
    /// - `None` means "do not change"
    /// - `Some(None)` means "clear note"
    /// - `Some(Some(v))` means "set note to v"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Option<String>>,
}

impl DomainMetadataUpdate {
    /// Applies this update to an existing metadata object.
    pub fn apply_to(&self, metadata: &mut DomainMetadata) {
        if let Some(is_favorite) = self.is_favorite {
            metadata.is_favorite = is_favorite;
        }
        if let Some(ref tags) = self.tags {
            metadata.tags.clone_from(tags);
        }
        if let Some(ref color) = self.color {
            metadata.color.clone_from(color);
        }
        if let Some(ref note) = self.note {
            metadata.note.clone_from(note);
        }
        metadata.touch();
    }
}

/// One batch tag operation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagRequest {
    /// Account ID.
    pub account_id: String,
    /// Domain ID.
    pub domain_id: String,
    /// Tags to add/remove/set based on the operation.
    pub tags: Vec<String>,
}

/// Result of a batch tag operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagResult {
    /// Number of successful items.
    pub success_count: usize,
    /// Number of failed items.
    pub failed_count: usize,
    /// Failure details.
    pub failures: Vec<BatchTagFailure>,
}

/// One batch tag operation failure item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTagFailure {
    /// Account ID.
    pub account_id: String,
    /// Domain ID.
    pub domain_id: String,
    /// Failure reason.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_metadata_is_empty() {
        let m = DomainMetadata::default();
        assert!(m.is_empty());
    }

    #[test]
    fn metadata_with_favorite_not_empty() {
        let m = DomainMetadata {
            is_favorite: true,
            ..DomainMetadata::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn metadata_with_tag_not_empty() {
        let m = DomainMetadata {
            tags: vec!["important".to_string()],
            ..DomainMetadata::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn metadata_with_color_not_empty() {
        let m = DomainMetadata {
            color: "red".to_string(),
            ..DomainMetadata::default()
        };
        assert!(!m.is_empty());
    }

    #[test]
    fn metadata_with_color_none_is_empty() {
        let m = DomainMetadata::default();
        assert_eq!(m.color, "none");
        assert!(m.is_empty());
    }

    #[test]
    fn metadata_touch_updates_timestamp() {
        let mut m = DomainMetadata::default();
        let before = m.updated_at;
        // `chrono::Utc::now()` precision allows these two calls to differ.
        std::thread::sleep(std::time::Duration::from_millis(2));
        m.touch();
        assert!(m.updated_at >= before);
    }

    #[test]
    fn metadata_key_storage_roundtrip() {
        let key = DomainMetadataKey::new("acc-123".to_string(), "domain.com".to_string());
        let storage = key.to_storage_key();
        assert_eq!(storage, "acc-123::domain.com");

        let parsed = DomainMetadataKey::from_storage_key(&storage).unwrap();
        assert_eq!(parsed, key);
    }

    #[test]
    fn metadata_key_invalid_storage_key() {
        assert!(DomainMetadataKey::from_storage_key("no-separator").is_none());
        assert!(DomainMetadataKey::from_storage_key("a::b::c").is_none());
    }

    #[test]
    fn apply_to_partial_update() {
        let mut m = DomainMetadata {
            tags: vec!["old".to_string()],
            color: "red".to_string(),
            ..DomainMetadata::default()
        };

        let update = DomainMetadataUpdate {
            is_favorite: Some(true),
            tags: None,
            color: None,
            note: None,
        };
        update.apply_to(&mut m);

        assert!(m.is_favorite);
        // Fields not present in the update should stay unchanged.
        assert_eq!(m.tags, vec!["old".to_string()]);
        assert_eq!(m.color, "red");
    }

    #[test]
    fn apply_to_full_update() {
        let mut m = DomainMetadata::default();

        let update = DomainMetadataUpdate {
            is_favorite: Some(true),
            tags: Some(vec!["a".to_string(), "b".to_string()]),
            color: Some("blue".to_string()),
            note: Some(Some("hello".to_string())),
        };
        update.apply_to(&mut m);

        assert!(m.is_favorite);
        assert_eq!(m.tags, vec!["a", "b"]);
        assert_eq!(m.color, "blue");
        assert_eq!(m.note, Some("hello".to_string()));
    }

    #[test]
    fn metadata_serde_roundtrip() {
        let m = DomainMetadata::new(
            true,
            vec!["tag1".to_string(), "tag2".to_string()],
            "green".to_string(),
            Some("a note".to_string()),
            Some(chrono::Utc::now()),
        );
        let json = serde_json::to_string(&m).unwrap();
        let deserialized: DomainMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(m, deserialized);
    }
}
