//! Encryption module.
//!
//! Provides AES-256-GCM encryption/decryption for account import/export payloads.

mod versions;

pub use versions::{CURRENT_FILE_VERSION, get_current_iterations, get_pbkdf2_iterations};

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use pbkdf2::pbkdf2_hmac_array;
use rand::RngCore;
use sha2::Sha256;

use crate::error::{CoreError, CoreResult};

// Resolve iteration count from version configuration.
const PBKDF2_ITERATIONS: u32 = get_current_iterations();
const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;
const KEY_LENGTH: usize = 32; // AES-256

/// Derives an encryption key from password, salt, and PBKDF2 iterations.
fn derive_key_with_iterations(password: &str, salt: &[u8], iterations: u32) -> [u8; KEY_LENGTH] {
    pbkdf2_hmac_array::<Sha256, KEY_LENGTH>(password.as_bytes(), salt, iterations)
}

/// Derives an encryption key using the default PBKDF2 iteration count.
fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    derive_key_with_iterations(password, salt, PBKDF2_ITERATIONS)
}

/// Encrypts plaintext bytes with password-based AES-256-GCM.
///
/// # Arguments
/// * `plaintext` - Plaintext bytes.
/// * `password` - Encryption password.
///
/// # Returns
/// Returns a tuple of (`salt_base64`, `nonce_base64`, `ciphertext_base64`).
pub fn encrypt(plaintext: &[u8], password: &str) -> CoreResult<(String, String, String)> {
    // Generate random salt and nonce.
    let mut salt = [0u8; SALT_LENGTH];
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce_bytes);

    // Derive key from password and salt.
    let key = derive_key(password, &salt);

    // Create encryptor instance.
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create cipher: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt plaintext.
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::SerializationError(format!("Encryption failed: {e}")))?;

    Ok((
        BASE64.encode(salt),
        BASE64.encode(nonce_bytes),
        BASE64.encode(ciphertext),
    ))
}

/// Decrypts Base64-encoded ciphertext with default iteration settings.
///
/// # Arguments
/// * `ciphertext_b64` - Base64-encoded ciphertext.
/// * `password` - Decryption password.
/// * `salt_b64` - Base64-encoded salt.
/// * `nonce_b64` - Base64-encoded nonce.
///
/// # Returns
/// Decrypted plaintext bytes.
pub fn decrypt(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
) -> CoreResult<Vec<u8>> {
    decrypt_with_iterations(
        ciphertext_b64,
        password,
        salt_b64,
        nonce_b64,
        PBKDF2_ITERATIONS,
    )
}

/// Decrypts data using a custom PBKDF2 iteration count (for backward compatibility).
///
/// # Arguments
/// * `ciphertext_b64` - Base64-encoded ciphertext.
/// * `password` - Decryption password.
/// * `salt_b64` - Base64-encoded salt.
/// * `nonce_b64` - Base64-encoded nonce.
/// * `iterations` - PBKDF2 iteration count.
///
/// # Returns
/// Decrypted plaintext bytes.
pub fn decrypt_with_iterations(
    ciphertext_b64: &str,
    password: &str,
    salt_b64: &str,
    nonce_b64: &str,
    iterations: u32,
) -> CoreResult<Vec<u8>> {
    // Decode Base64 payload parts.
    let salt = BASE64
        .decode(salt_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid salt: {e}")))?;
    let nonce_bytes = BASE64
        .decode(nonce_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid nonce: {e}")))?;
    let ciphertext = BASE64
        .decode(ciphertext_b64)
        .map_err(|e| CoreError::SerializationError(format!("Invalid ciphertext: {e}")))?;

    // Derive key using the provided iteration count.
    let key = derive_key_with_iterations(password, &salt, iterations);

    // Create decryptor instance.
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create cipher: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Decrypt ciphertext.
    cipher.decrypt(nonce, ciphertext.as_ref()).map_err(|_| {
        CoreError::SerializationError(
            "Decryption failed: invalid password or corrupted data".to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let plaintext = b"hello, DNS orchestrator!";
        let password = "strong-password-123";

        let (salt, nonce, ciphertext) = encrypt(plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, password, &salt, &nonce).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_wrong_password_fails() {
        let plaintext = b"secret data";
        let (salt, nonce, ciphertext) = encrypt(plaintext, "correct-password").unwrap();

        let result = decrypt(&ciphertext, "wrong-password", &salt, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_corrupted_ciphertext_fails() {
        let plaintext = b"some data";
        let password = "password";
        let (salt, nonce, _) = encrypt(plaintext, password).unwrap();

        // Construct valid Base64 data that is not valid ciphertext.
        let corrupted = BASE64.encode(b"this is not valid ciphertext at all!!");
        let result = decrypt(&corrupted, password, &salt, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_invalid_base64_fails() {
        let result = decrypt("not-valid-base64!!!", "password", "also-bad!!!", "bad!!!");
        assert!(result.is_err());
    }

    #[test]
    fn encrypt_produces_different_output() {
        let plaintext = b"same data";
        let password = "same-password";

        let (salt1, nonce1, ct1) = encrypt(plaintext, password).unwrap();
        let (salt2, nonce2, ct2) = encrypt(plaintext, password).unwrap();

        // Random salt/nonce should produce different output.
        assert!(salt1 != salt2 || nonce1 != nonce2 || ct1 != ct2);
    }

    #[test]
    fn decrypt_with_different_iterations() {
        let plaintext = b"version test data";
        let password = "test-password";

        // Encrypt using current version settings (v2, 600k iterations).
        let (salt, nonce, ciphertext) = encrypt(plaintext, password).unwrap();

        // Decrypt with v1 iterations (100k) -> different key, should fail.
        let v1_iterations = versions::get_pbkdf2_iterations(1).unwrap();
        let result = decrypt_with_iterations(&ciphertext, password, &salt, &nonce, v1_iterations);
        assert!(result.is_err());
    }
}
