//! 加密模块
//!
//! 使用 AES-256-GCM 加密凭证

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

use crate::error::ApiError;

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

/// 加密管理器
#[derive(Clone)]
pub struct CryptoManager {
    key: [u8; KEY_SIZE],
}

impl CryptoManager {
    /// 从 hex 编码的密钥创建
    pub fn from_hex_key(hex_key: &str) -> Result<Self, ApiError> {
        let key_bytes = hex::decode(hex_key)
            .map_err(|e| ApiError::Encryption(format!("无效的密钥格式: {e}")))?;

        if key_bytes.len() != KEY_SIZE {
            return Err(ApiError::Encryption(format!(
                "密钥长度必须是 {KEY_SIZE} 字节（{} 个 hex 字符）",
                KEY_SIZE * 2
            )));
        }

        let mut key = [0u8; KEY_SIZE];
        key.copy_from_slice(&key_bytes);
        Ok(Self { key })
    }

    /// 生成随机密钥（hex 编码）
    pub fn generate_key() -> String {
        let mut key = [0u8; KEY_SIZE];
        rand::rng().fill_bytes(&mut key);
        hex::encode(key)
    }

    /// 加密数据
    pub fn encrypt(&self, plaintext: &str) -> Result<String, ApiError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| ApiError::Encryption(format!("创建加密器失败: {e}")))?;

        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| ApiError::Encryption(format!("加密失败: {e}")))?;

        // 格式: base64(nonce || ciphertext)
        let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(combined))
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted: &str) -> Result<String, ApiError> {
        let combined = BASE64
            .decode(encrypted)
            .map_err(|e| ApiError::Encryption(format!("Base64 解码失败: {e}")))?;

        if combined.len() < NONCE_SIZE {
            return Err(ApiError::Encryption("加密数据格式无效".to_string()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| ApiError::Encryption(format!("创建解密器失败: {e}")))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ApiError::Encryption(format!("解密失败: {e}")))?;

        String::from_utf8(plaintext)
            .map_err(|e| ApiError::Encryption(format!("UTF-8 解码失败: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = CryptoManager::generate_key();
        let crypto = CryptoManager::from_hex_key(&key).unwrap();

        let plaintext = r#"{"apiToken":"test-token-12345"}"#;
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
