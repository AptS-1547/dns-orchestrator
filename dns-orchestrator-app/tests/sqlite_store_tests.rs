#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! Integration tests for `SqliteStore` — covers `AccountRepository`,
//! `CredentialStore`, and `DomainMetadataRepository` trait implementations.

use std::collections::HashMap;

use dns_orchestrator_app::adapters::SqliteStore;
use dns_orchestrator_core::error::CoreError;
use dns_orchestrator_core::traits::{AccountRepository, CredentialStore, DomainMetadataRepository};
use dns_orchestrator_core::types::{
    Account, AccountStatus, DomainMetadata, DomainMetadataKey, DomainMetadataUpdate,
    ProviderCredentials, ProviderType,
};

// ===== Helpers =====

const TEST_PASSWORD: &str = "test-encryption-password-32chars!";

async fn create_test_store() -> (SqliteStore, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("test.db");
    let store = SqliteStore::new(&db_path, Some(TEST_PASSWORD.to_string()))
        .await
        .expect("failed to create SqliteStore");
    (store, tmp)
}

async fn create_test_store_no_password() -> (SqliteStore, tempfile::TempDir) {
    let tmp = tempfile::tempdir().expect("failed to create temp dir");
    let db_path = tmp.path().join("test.db");
    let store = SqliteStore::new(&db_path, None)
        .await
        .expect("failed to create SqliteStore");
    (store, tmp)
}

fn make_account(id: &str) -> Account {
    Account {
        id: id.to_string(),
        name: format!("Test Account {id}"),
        provider: ProviderType::Cloudflare,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        status: None,
        error: None,
    }
}

fn test_credentials() -> ProviderCredentials {
    ProviderCredentials::Cloudflare {
        api_token: "test-token-12345".to_string(),
    }
}

fn make_key(account_id: &str, domain_id: &str) -> DomainMetadataKey {
    DomainMetadataKey::new(account_id.to_string(), domain_id.to_string())
}

fn make_metadata(is_favorite: bool, tags: Vec<&str>, color: &str) -> DomainMetadata {
    DomainMetadata::new(
        is_favorite,
        tags.into_iter().map(String::from).collect(),
        color.to_string(),
        None,
        if is_favorite {
            Some(chrono::Utc::now())
        } else {
            None
        },
    )
}

// ===== AccountRepository Tests =====

#[tokio::test]
async fn account_find_all_empty() {
    let (store, _tmp) = create_test_store().await;
    let accounts = AccountRepository::find_all(&store).await.unwrap();
    assert!(accounts.is_empty());
}

#[tokio::test]
async fn account_save_and_find_by_id() {
    let (store, _tmp) = create_test_store().await;
    let account = make_account("acc-1");
    AccountRepository::save(&store, &account).await.unwrap();

    let found = AccountRepository::find_by_id(&store, "acc-1")
        .await
        .unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, "acc-1");
    assert_eq!(found.name, "Test Account acc-1");
}

#[tokio::test]
async fn account_find_by_id_not_found() {
    let (store, _tmp) = create_test_store().await;
    let found = AccountRepository::find_by_id(&store, "nonexistent")
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn account_save_upsert_updates_existing() {
    let (store, _tmp) = create_test_store().await;
    let mut account = make_account("acc-1");
    AccountRepository::save(&store, &account).await.unwrap();

    account.name = "Updated Name".to_string();
    AccountRepository::save(&store, &account).await.unwrap();

    let found = AccountRepository::find_by_id(&store, "acc-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.name, "Updated Name");

    let all = AccountRepository::find_all(&store).await.unwrap();
    assert_eq!(all.len(), 1);
}

#[tokio::test]
async fn account_delete_existing() {
    let (store, _tmp) = create_test_store().await;
    AccountRepository::save(&store, &make_account("acc-1"))
        .await
        .unwrap();
    AccountRepository::delete(&store, "acc-1").await.unwrap();

    let found = AccountRepository::find_by_id(&store, "acc-1")
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn account_delete_nonexistent_returns_error() {
    let (store, _tmp) = create_test_store().await;
    let result = AccountRepository::delete(&store, "nonexistent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CoreError::AccountNotFound(_)));
}

#[tokio::test]
async fn account_save_all_batch() {
    let (store, _tmp) = create_test_store().await;
    let accounts = vec![make_account("a1"), make_account("a2"), make_account("a3")];
    AccountRepository::save_all(&store, &accounts)
        .await
        .unwrap();

    let all = AccountRepository::find_all(&store).await.unwrap();
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn account_update_status_existing() {
    let (store, _tmp) = create_test_store().await;
    AccountRepository::save(&store, &make_account("acc-1"))
        .await
        .unwrap();

    AccountRepository::update_status(
        &store,
        "acc-1",
        AccountStatus::Error,
        Some("bad creds".into()),
    )
    .await
    .unwrap();

    let found = AccountRepository::find_by_id(&store, "acc-1")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.status, Some(AccountStatus::Error));
    assert_eq!(found.error, Some("bad creds".to_string()));
}

#[tokio::test]
async fn account_update_status_nonexistent_returns_error() {
    let (store, _tmp) = create_test_store().await;
    let result =
        AccountRepository::update_status(&store, "nonexistent", AccountStatus::Active, None).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CoreError::AccountNotFound(_)));
}

// ===== CredentialStore Tests =====

#[tokio::test]
async fn credential_load_all_empty() {
    let (store, _tmp) = create_test_store().await;
    let creds = CredentialStore::load_all(&store).await.unwrap();
    assert!(creds.is_empty());
}

#[tokio::test]
async fn credential_set_and_get() {
    let (store, _tmp) = create_test_store().await;
    let creds = test_credentials();
    CredentialStore::set(&store, "acc-1", &creds).await.unwrap();

    let loaded = CredentialStore::get(&store, "acc-1").await.unwrap();
    assert!(loaded.is_some());
    match loaded.unwrap() {
        ProviderCredentials::Cloudflare { api_token } => {
            assert_eq!(api_token, "test-token-12345");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Cloudflare credentials"),
    }
}

#[tokio::test]
async fn credential_get_nonexistent() {
    let (store, _tmp) = create_test_store().await;
    let loaded = CredentialStore::get(&store, "nonexistent").await.unwrap();
    assert!(loaded.is_none());
}

#[tokio::test]
async fn credential_set_upsert_overwrites() {
    let (store, _tmp) = create_test_store().await;
    let creds1 = ProviderCredentials::Cloudflare {
        api_token: "token-1".to_string(),
    };
    let creds2 = ProviderCredentials::Cloudflare {
        api_token: "token-2".to_string(),
    };

    CredentialStore::set(&store, "acc-1", &creds1)
        .await
        .unwrap();
    CredentialStore::set(&store, "acc-1", &creds2)
        .await
        .unwrap();

    let loaded = CredentialStore::get(&store, "acc-1")
        .await
        .unwrap()
        .unwrap();
    match loaded {
        ProviderCredentials::Cloudflare { api_token } => {
            assert_eq!(api_token, "token-2");
        }
        #[allow(unreachable_patterns)]
        _ => panic!("Expected Cloudflare credentials"),
    }
}

#[tokio::test]
async fn credential_remove_existing() {
    let (store, _tmp) = create_test_store().await;
    CredentialStore::set(&store, "acc-1", &test_credentials())
        .await
        .unwrap();
    CredentialStore::remove(&store, "acc-1").await.unwrap();

    let loaded = CredentialStore::get(&store, "acc-1").await.unwrap();
    assert!(loaded.is_none());
}

#[tokio::test]
async fn credential_remove_nonexistent_no_error() {
    let (store, _tmp) = create_test_store().await;
    CredentialStore::remove(&store, "nonexistent")
        .await
        .unwrap();
}

#[tokio::test]
async fn credential_save_all_replaces() {
    let (store, _tmp) = create_test_store().await;
    CredentialStore::set(&store, "old-acc", &test_credentials())
        .await
        .unwrap();

    let mut new_creds = HashMap::new();
    new_creds.insert("new-1".to_string(), test_credentials());
    new_creds.insert("new-2".to_string(), test_credentials());
    CredentialStore::save_all(&store, &new_creds).await.unwrap();

    let all = CredentialStore::load_all(&store).await.unwrap();
    assert_eq!(all.len(), 2);
    assert!(all.contains_key("new-1"));
    assert!(all.contains_key("new-2"));
    assert!(!all.contains_key("old-acc"));
}

#[tokio::test]
async fn credential_raw_json_roundtrip() {
    let (store, _tmp) = create_test_store().await;
    CredentialStore::set(&store, "acc-1", &test_credentials())
        .await
        .unwrap();

    let raw = CredentialStore::load_raw_json(&store).await.unwrap();
    let parsed: HashMap<String, serde_json::Value> = serde_json::from_str(&raw).unwrap();
    assert!(parsed.contains_key("acc-1"));

    // Import into a fresh store
    let (store2, _tmp2) = create_test_store().await;
    CredentialStore::save_raw_json(&store2, &raw).await.unwrap();

    let loaded = CredentialStore::get(&store2, "acc-1").await.unwrap();
    assert!(loaded.is_some());
}

#[tokio::test]
async fn credential_error_when_no_encryption_password() {
    let (store, _tmp) = create_test_store_no_password().await;
    let result = CredentialStore::set(&store, "acc-1", &test_credentials()).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CoreError::CredentialError(_)));
}

#[tokio::test]
async fn credential_load_all_empty_without_password_ok() {
    let (store, _tmp) = create_test_store_no_password().await;
    let result = CredentialStore::load_all(&store).await.unwrap();
    assert!(result.is_empty());
}

// ===== DomainMetadataRepository Tests =====

#[tokio::test]
async fn metadata_find_by_key_empty() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "example.com");
    let result = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn metadata_save_and_find_by_key() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "example.com");
    let metadata = make_metadata(true, vec!["important"], "red");
    DomainMetadataRepository::save(&store, &key, &metadata)
        .await
        .unwrap();

    let found = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap()
        .unwrap();
    assert!(found.is_favorite);
    assert_eq!(found.tags, vec!["important"]);
    assert_eq!(found.color, "red");
}

#[tokio::test]
async fn metadata_save_empty_deletes() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "example.com");
    DomainMetadataRepository::save(&store, &key, &make_metadata(true, vec!["tag"], "blue"))
        .await
        .unwrap();

    let empty = DomainMetadata::default();
    assert!(empty.is_empty());
    DomainMetadataRepository::save(&store, &key, &empty)
        .await
        .unwrap();

    let found = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn metadata_find_by_keys_batch() {
    let (store, _tmp) = create_test_store().await;
    let k1 = make_key("acc-1", "a.com");
    let k2 = make_key("acc-1", "b.com");
    let k3 = make_key("acc-1", "c.com");

    DomainMetadataRepository::save(&store, &k1, &make_metadata(true, vec![], "red"))
        .await
        .unwrap();
    DomainMetadataRepository::save(&store, &k2, &make_metadata(false, vec!["x"], "none"))
        .await
        .unwrap();

    let result = DomainMetadataRepository::find_by_keys(&store, &[k1.clone(), k2.clone(), k3])
        .await
        .unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.contains_key(&k1));
    assert!(result.contains_key(&k2));
}

#[tokio::test]
async fn metadata_find_by_keys_empty_input() {
    let (store, _tmp) = create_test_store().await;
    let result = DomainMetadataRepository::find_by_keys(&store, &[])
        .await
        .unwrap();
    assert!(result.is_empty());
}

#[tokio::test]
async fn metadata_batch_save() {
    let (store, _tmp) = create_test_store().await;
    let entries = vec![
        (
            make_key("acc-1", "a.com"),
            make_metadata(true, vec![], "red"),
        ),
        (
            make_key("acc-1", "b.com"),
            make_metadata(false, vec!["tag1"], "blue"),
        ),
    ];
    DomainMetadataRepository::batch_save(&store, &entries)
        .await
        .unwrap();

    assert!(
        DomainMetadataRepository::find_by_key(&store, &make_key("acc-1", "a.com"))
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        DomainMetadataRepository::find_by_key(&store, &make_key("acc-1", "b.com"))
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn metadata_update_partial() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "example.com");
    DomainMetadataRepository::save(&store, &key, &make_metadata(false, vec!["old"], "red"))
        .await
        .unwrap();

    let update = DomainMetadataUpdate {
        is_favorite: Some(true),
        tags: None,
        color: None,
        note: None,
    };
    DomainMetadataRepository::update(&store, &key, &update)
        .await
        .unwrap();

    let found = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap()
        .unwrap();
    assert!(found.is_favorite);
    assert_eq!(found.tags, vec!["old"]);
    assert_eq!(found.color, "red");
}

#[tokio::test]
async fn metadata_update_nonexistent_creates() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "new.com");

    let update = DomainMetadataUpdate {
        is_favorite: Some(true),
        tags: Some(vec!["fresh".to_string()]),
        color: None,
        note: None,
    };
    DomainMetadataRepository::update(&store, &key, &update)
        .await
        .unwrap();

    let found = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap()
        .unwrap();
    assert!(found.is_favorite);
    assert_eq!(found.tags, vec!["fresh"]);
}

#[tokio::test]
async fn metadata_delete() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "example.com");
    DomainMetadataRepository::save(&store, &key, &make_metadata(true, vec![], "red"))
        .await
        .unwrap();

    DomainMetadataRepository::delete(&store, &key)
        .await
        .unwrap();
    let found = DomainMetadataRepository::find_by_key(&store, &key)
        .await
        .unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn metadata_delete_nonexistent_no_error() {
    let (store, _tmp) = create_test_store().await;
    let key = make_key("acc-1", "nonexistent.com");
    DomainMetadataRepository::delete(&store, &key)
        .await
        .unwrap();
}

#[tokio::test]
async fn metadata_delete_by_account() {
    let (store, _tmp) = create_test_store().await;
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "a.com"),
        &make_metadata(true, vec![], "red"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "b.com"),
        &make_metadata(false, vec![], "blue"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-2", "c.com"),
        &make_metadata(true, vec![], "green"),
    )
    .await
    .unwrap();

    DomainMetadataRepository::delete_by_account(&store, "acc-1")
        .await
        .unwrap();

    assert!(
        DomainMetadataRepository::find_by_key(&store, &make_key("acc-1", "a.com"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        DomainMetadataRepository::find_by_key(&store, &make_key("acc-1", "b.com"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        DomainMetadataRepository::find_by_key(&store, &make_key("acc-2", "c.com"))
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn metadata_find_favorites_by_account() {
    let (store, _tmp) = create_test_store().await;
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "fav.com"),
        &make_metadata(true, vec![], "none"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "nonfav.com"),
        &make_metadata(false, vec!["x"], "red"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-2", "other.com"),
        &make_metadata(true, vec![], "none"),
    )
    .await
    .unwrap();

    let favs = DomainMetadataRepository::find_favorites_by_account(&store, "acc-1")
        .await
        .unwrap();
    assert_eq!(favs.len(), 1);
    assert_eq!(favs[0].domain_id, "fav.com");
}

#[tokio::test]
async fn metadata_find_by_tag() {
    let (store, _tmp) = create_test_store().await;
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "a.com"),
        &make_metadata(false, vec!["prod", "important"], "none"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "b.com"),
        &make_metadata(false, vec!["staging"], "none"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-2", "c.com"),
        &make_metadata(false, vec!["prod"], "none"),
    )
    .await
    .unwrap();

    let results = DomainMetadataRepository::find_by_tag(&store, "prod")
        .await
        .unwrap();
    assert_eq!(results.len(), 2);

    let mut domain_ids: Vec<&str> = results.iter().map(|k| k.domain_id.as_str()).collect();
    domain_ids.sort_unstable();
    assert_eq!(domain_ids, vec!["a.com", "c.com"]);
}

#[tokio::test]
async fn metadata_list_all_tags() {
    let (store, _tmp) = create_test_store().await;
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "a.com"),
        &make_metadata(false, vec!["beta", "alpha"], "none"),
    )
    .await
    .unwrap();
    DomainMetadataRepository::save(
        &store,
        &make_key("acc-1", "b.com"),
        &make_metadata(false, vec!["alpha", "gamma"], "none"),
    )
    .await
    .unwrap();

    let tags = DomainMetadataRepository::list_all_tags(&store)
        .await
        .unwrap();
    assert_eq!(tags, vec!["alpha", "beta", "gamma"]);
}

#[tokio::test]
async fn metadata_list_all_tags_empty() {
    let (store, _tmp) = create_test_store().await;
    let tags = DomainMetadataRepository::list_all_tags(&store)
        .await
        .unwrap();
    assert!(tags.is_empty());
}
