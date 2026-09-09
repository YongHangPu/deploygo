use std::collections::HashMap;

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::config::{Project, ServerConfig};
use crate::error::AppError;

const BACKUP_MEMORY_KIB: u32 = 64 * 1024;
const BACKUP_ITERATIONS: u32 = 3;
const BACKUP_PARALLELISM: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub exported_at: String,
    pub servers: Vec<ServerConfig>,
    pub projects: Vec<Project>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credential_payload: Option<EncryptedCredentialPayload>,

    // v2 兼容字段：导入时仍可识别，但 v3 导出时不会再写入。
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub passwords: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub jump_passwords: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<String>,
}

impl ExportData {
    pub fn v3(
        servers: Vec<ServerConfig>,
        projects: Vec<Project>,
        credential_payload: Option<EncryptedCredentialPayload>,
    ) -> Self {
        Self {
            version: 3,
            exported_at: chrono::Utc::now().to_rfc3339(),
            servers,
            projects,
            credential_mode: Some(if credential_payload.is_some() {
                "password_encrypted".to_string()
            } else {
                "excluded".to_string()
            }),
            credential_payload,
            passwords: HashMap::new(),
            jump_passwords: HashMap::new(),
            encryption_key: None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialData {
    #[serde(default)]
    pub passwords: HashMap<String, String>,
    #[serde(default)]
    pub jump_passwords: HashMap<String, String>,
}

impl Drop for CredentialData {
    fn drop(&mut self) {
        for value in self.passwords.values_mut() {
            value.zeroize();
        }
        for value in self.jump_passwords.values_mut() {
            value.zeroize();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedCredentialPayload {
    pub kdf: BackupKdf,
    pub cipher: BackupCipher,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupKdf {
    pub name: String,
    pub salt: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupCipher {
    pub name: String,
    pub nonce: String,
    pub ciphertext: String,
}

pub fn encrypt_credentials(
    credentials: &CredentialData,
    password: &str,
) -> Result<EncryptedCredentialPayload, AppError> {
    encrypt_credentials_with_params(
        credentials,
        password,
        BACKUP_MEMORY_KIB,
        BACKUP_ITERATIONS,
        BACKUP_PARALLELISM,
    )
}

fn encrypt_credentials_with_params(
    credentials: &CredentialData,
    password: &str,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<EncryptedCredentialPayload, AppError> {
    validate_backup_password(password)?;

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    let key = Zeroizing::new(derive_key(password, &salt, memory_kib, iterations, parallelism)?);
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| AppError {
        message: "无法初始化备份加密器".to_string(),
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let plaintext = Zeroizing::new(serde_json::to_vec(credentials)?);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_slice())
        .map_err(|_| AppError {
            message: "凭据备份加密失败".to_string(),
        })?;

    Ok(EncryptedCredentialPayload {
        kdf: BackupKdf {
            name: "argon2id".to_string(),
            salt: encode(&salt),
            memory_kib,
            iterations,
            parallelism,
        },
        cipher: BackupCipher {
            name: "aes-256-gcm".to_string(),
            nonce: encode(&nonce),
            ciphertext: encode(&ciphertext),
        },
    })
}

pub fn decrypt_credentials(
    payload: &EncryptedCredentialPayload,
    password: &str,
) -> Result<CredentialData, AppError> {
    if payload.kdf.name != "argon2id" || payload.cipher.name != "aes-256-gcm" {
        return Err(AppError {
            message: "不支持的凭据备份加密格式".to_string(),
        });
    }
    validate_kdf_limits(&payload.kdf)?;

    let salt = decode(&payload.kdf.salt, "KDF salt")?;
    if salt.len() != 16 {
        return Err(AppError {
            message: "凭据备份 salt 长度无效".to_string(),
        });
    }
    let nonce = decode(&payload.cipher.nonce, "AES-GCM nonce")?;
    if nonce.len() != 12 {
        return Err(AppError {
            message: "凭据备份 nonce 长度无效".to_string(),
        });
    }
    let ciphertext = decode(&payload.cipher.ciphertext, "密文")?;

    let key = Zeroizing::new(derive_key(
        password,
        &salt,
        payload.kdf.memory_kib,
        payload.kdf.iterations,
        payload.kdf.parallelism,
    )?);
    let cipher = Aes256Gcm::new_from_slice(key.as_ref()).map_err(|_| AppError {
        message: "无法初始化备份解密器".to_string(),
    })?;
    let plaintext = Zeroizing::new(cipher
        .decrypt(aes_gcm::Nonce::from_slice(&nonce), ciphertext.as_slice())
        .map_err(|_| AppError {
            message: "备份密码错误或凭据数据已损坏".to_string(),
        })?);

    serde_json::from_slice(&plaintext).map_err(|_| AppError {
        message: "凭据备份内容格式无效".to_string(),
    })
}

pub fn validate_backup_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < 8 {
        return Err(AppError {
            message: "备份密码至少需要 8 个字符".to_string(),
        });
    }
    Ok(())
}

fn validate_kdf_limits(kdf: &BackupKdf) -> Result<(), AppError> {
    if !(8 * 1024..=256 * 1024).contains(&kdf.memory_kib)
        || !(1..=10).contains(&kdf.iterations)
        || !(1..=8).contains(&kdf.parallelism)
    {
        return Err(AppError {
            message: "备份中的 Argon2id 参数超出安全范围".to_string(),
        });
    }
    Ok(())
}

fn derive_key(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; 32], AppError> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(32)).map_err(|error| {
        AppError {
            message: format!("Argon2id 参数无效: {}", error),
        }
    })?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|error| AppError {
            message: format!("备份密钥派生失败: {}", error),
        })?;
    Ok(key)
}

fn encode(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}

fn decode(value: &str, label: &str) -> Result<Vec<u8>, AppError> {
    base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(value)
        .map_err(|_| AppError {
            message: format!("凭据备份中的 {} 格式无效", label),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_credentials() -> CredentialData {
        CredentialData {
            passwords: HashMap::from([("server-1".to_string(), "secret".to_string())]),
            jump_passwords: HashMap::from([("server-2".to_string(), "jump-secret".to_string())]),
        }
    }

    #[test]
    fn encrypted_credentials_roundtrip() {
        let credentials = sample_credentials();
        let payload = encrypt_credentials_with_params(
            &credentials,
            "correct horse battery staple",
            8 * 1024,
            1,
            1,
        )
        .unwrap();

        assert_eq!(
            decrypt_credentials(&payload, "correct horse battery staple").unwrap(),
            credentials
        );
    }

    #[test]
    fn wrong_password_is_rejected() {
        let payload = encrypt_credentials_with_params(
            &sample_credentials(),
            "correct-password",
            8 * 1024,
            1,
            1,
        )
        .unwrap();

        let error = decrypt_credentials(&payload, "wrong-password").unwrap_err();
        assert!(error.message.contains("密码错误"));
    }

    #[test]
    fn v3_without_credentials_has_no_local_encryption_key() {
        let export = ExportData::v3(Vec::new(), Vec::new(), None);
        let json = serde_json::to_value(export).unwrap();
        assert_eq!(json["version"], 3);
        assert_eq!(json["credential_mode"], "excluded");
        assert!(json.get("credential_payload").is_none());
        assert!(json.get("encryption_key").is_none());
        assert!(json.get("passwords").is_none());
    }
}
