//! DNS record-related types.

use serde::{Deserialize, Serialize};

/// Request payload for deleting DNS records in batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    /// Domain ID.
    pub domain_id: String,
    /// Record ID list.
    pub record_ids: Vec<String>,
}
