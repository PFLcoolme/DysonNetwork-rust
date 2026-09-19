//! 端到端加密 (E2EE) 模块
//!
//! 提供文件加密和解密功能

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD as ENGINE, Engine};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// 加密文件元数据
#[derive(Debug, Clone)]
pub struct EncryptedFile {
    pub ciphertext: Vec<u8>,
    pub iv: Vec<u8>,
    pub auth_tag: Vec<u8>,
    pub encryption_key_id: String,
}

/// 加密密钥管理
pub struct EncryptionKeyManager {
    /// 密钥存储 (实际应从数据库或密钥管理服务获取)
    keys: std::collections::HashMap<String, Vec<u8>>,
}

impl EncryptionKeyManager {
    pub fn new() -> Self {
        Self {
            keys: std::collections::HashMap::new(),
        }
    }

    /// 生成新的加密密钥
    pub fn generate_key(&self) -> (String, String) {
        let key_id = Uuid::new_v4().to_string();
        let key = (0..32).map(|_| rand::random::<u8>()).collect::<Vec<_>>();

        let key_b64 = ENGINE.encode(&key);
        // 实际实现中应该存储到数据库
        (key_id, key_b64)
    }

    /// 获取密钥
    pub fn get_key(&self, key_id: &str) -> Option<Vec<u8>> {
        self.keys.get(key_id).cloned()
    }
}

impl Default for EncryptionKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算文件 checksum
pub fn calculate_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// 加密文件数据
pub fn encrypt_file_data(
    plaintext: &[u8],
    encryption_key: &[u8],
) -> Result<EncryptedFile> {
    if encryption_key.len() != 32 {
        anyhow::bail!("Invalid key length");
    }

    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);

    let nonce = Aes256Gcm::generate_nonce(&mut rand::rngs::OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext)?;
    
    // 提取 auth tag (last 16 bytes)
    let auth_tag = ciphertext[ciphertext.len() - 16..].to_vec();
    let ciphertext_only = &ciphertext[..ciphertext.len() - 16];
    
    Ok(EncryptedFile {
        ciphertext: ciphertext_only.to_vec(),
        iv: nonce.to_vec(),
        auth_tag,
        encryption_key_id: String::new(),
    })
}

/// 解密文件数据
pub fn decrypt_file_data(encrypted: &EncryptedFile, encryption_key: &[u8]) -> Result<Vec<u8>> {
    if encryption_key.len() != 32 {
        anyhow::bail!("Invalid key length");
    }

    let key = Key::<Aes256Gcm>::from_slice(encryption_key);
    let cipher = Aes256Gcm::new(key);

    let nonce = Nonce::from_slice(&encrypted.iv);

    // 重组 ciphertext + auth_tag
    let mut ciphertext_with_tag = encrypted.ciphertext.clone();
    ciphertext_with_tag.extend_from_slice(&encrypted.auth_tag);

    let plaintext = cipher.decrypt(nonce, ciphertext_with_tag.as_slice())?;
    Ok(plaintext)
}

/// 生成缩略图密钥 (用于图片)
pub fn generate_thumbnail_key(original_key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"thumbnail_");
    hasher.update(original_key);
    hasher.finalize().to_vec()
}

/// 检查文件是否加密
pub fn is_encrypted(data: &[u8]) -> bool {
    // 检查文件头标记
    data.starts_with(b"ENCRYPTED:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = vec![0u8; 32];
        let plaintext = b"Hello, World!";
        
        let encrypted = encrypt_file_data(plaintext, &key).unwrap();
        let decrypted = decrypt_file_data(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_checksum() {
        let data = b"test data";
        let checksum1 = calculate_checksum(data);
        let checksum2 = calculate_checksum(data);
        
        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA-256 = 256 bits = 64 hex chars
    }
}