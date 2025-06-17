use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use rand::RngCore;

use crate::error::{DatabaseError, Result};

/// 加密器，负责数据的加密和解密
#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for Crypto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Crypto")
            .field("cipher", &"<AES-GCM cipher>")
            .finish()
    }
}

impl Crypto {
    /// 从提供的密钥创建加密器
    pub fn new(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(DatabaseError::Encryption(
                "密钥长度必须为32字节".to_string(),
            ));
        }

        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// 生成随机密钥
    pub fn generate_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// 加密数据
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // 生成随机nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        // 加密数据
        let ciphertext = self
            .cipher
            .encrypt(&nonce, data)
            .map_err(|e| DatabaseError::Encryption(format!("加密失败: {}", e)))?;

        // 将nonce和密文组合
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// 解密数据
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(DatabaseError::Encryption(
                "加密数据太短，至少需要12字节".to_string(),
            ));
        }

        // 分离nonce和密文
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密数据
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| DatabaseError::Encryption(format!("解密失败: {}", e)))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = Crypto::generate_key();
        let crypto = Crypto::new(&key).unwrap();
        
        let data = b"Hello, World!";
        let encrypted = crypto.encrypt(data).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        
        assert_eq!(data, &decrypted[..]);
    }
} 