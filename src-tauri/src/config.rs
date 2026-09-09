use std::path::PathBuf;

use base64::Engine;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::error::AppError;

/// 认证信息：密码认证或 SSH 密钥认证。
/// 仅由完整版部署引擎（overlay 注入）使用。
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AuthMethod {
    Password(String),
    Key { path: String, passphrase: Option<String> },
}

impl Drop for AuthMethod {
    fn drop(&mut self) {
        match self {
            Self::Password(password) => password.zeroize(),
            Self::Key { passphrase, .. } => {
                if let Some(passphrase) = passphrase {
                    passphrase.zeroize();
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub local_dist_path: String,
    pub build_command: Option<String>,
    pub server_id: String,
    pub keep_versions: i32,
    pub live_root_path: String,
    pub releases_root_path: String,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: String, // 可选值："password" | "key"
    /// SSH 私钥路径（例如 `~/.ssh/id_ed25519`）。
    pub key_path: Option<String>,
    // ---- 跳板机 ----
    pub jump_host: Option<String>,
    pub jump_port: Option<u16>,
    pub jump_username: Option<String>,
    pub jump_auth_method: Option<String>, // 可选值："password" | "key"
    pub jump_key_path: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployRecord {
    pub id: i64,
    pub project_id: String,
    pub project_name: String,
    pub server_id: String,
    pub server_name: String,
    pub release_dir_path: String,
    pub live_root_path: String,
    pub version: Option<String>,
    pub artifact_fingerprint: Option<String>,
    pub status: String, // 可选值：success / failed / cancelled / rollback
    pub file_count: i64,
    pub total_size: i64,
    pub exit_code: Option<i64>,
    pub duration_ms: i64,
    pub log: String,
    pub created_at: String,
}

pub struct AppConfig {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
}

impl AppConfig {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.deploygo.app");

        let db_path = data_dir.join("deploygo.db");

        AppConfig {
            data_dir,
            db_path,
        }
    }

    pub fn ensure_data_dir(&self) -> Result<(), AppError> {
        std::fs::create_dir_all(&self.data_dir)?;
        Ok(())
    }
}

pub struct ConfigStore {
    config: AppConfig,
}

impl ConfigStore {
    pub fn new() -> Result<Self, AppError> {
        let config = AppConfig::new();
        config.ensure_data_dir()?;
        Ok(ConfigStore { config })
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }
}

pub fn derive_releases_root_path(live_root_path: &str) -> Result<String, AppError> {
    let (normalized, _) = normalize_absolute_remote_path(live_root_path)?;
    let separator = normalized.rfind('/').ok_or_else(|| AppError {
        message: format!("无法根据线上发布目录推导 releases 目录: {live_root_path}"),
    })?;
    let parent = &normalized[..separator];

    if parent.is_empty() {
        Ok("/releases".to_string())
    } else {
        Ok(format!("{parent}/releases"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemotePathStyle {
    Posix,
    WindowsDrive,
    Unc,
}

fn normalize_absolute_remote_path(path: &str) -> Result<(String, RemotePathStyle), AppError> {
    let raw = path.trim().replace('\\', "/");
    if raw.is_empty() {
        return Err(AppError {
            message: "线上发布目录不能为空".to_string(),
        });
    }
    if raw.chars().any(char::is_control) {
        return Err(AppError {
            message: "远端目录不能包含控制字符".to_string(),
        });
    }

    let (prefix, remainder, style) = if let Some(remainder) = raw.strip_prefix("//") {
        ("//".to_string(), remainder, RemotePathStyle::Unc)
    } else if raw.len() >= 3
        && raw.as_bytes()[0].is_ascii_alphabetic()
        && raw.as_bytes()[1] == b':'
        && raw.as_bytes()[2] == b'/'
    {
        (
            format!("{}:/", raw[..1].to_ascii_uppercase()),
            &raw[3..],
            RemotePathStyle::WindowsDrive,
        )
    } else if let Some(remainder) = raw.strip_prefix('/') {
        ("/".to_string(), remainder, RemotePathStyle::Posix)
    } else {
        return Err(AppError {
            message: format!("远端目录必须是 Linux、Windows 盘符或 UNC 绝对路径: {path}"),
        });
    };

    let segments = remainder
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.iter().any(|segment| matches!(*segment, "." | "..")) {
        return Err(AppError {
            message: format!("远端目录不能包含 . 或 .. 路径段: {path}"),
        });
    }

    let minimum_segments = if style == RemotePathStyle::Unc { 3 } else { 1 };
    if segments.len() < minimum_segments {
        let detail = match style {
            RemotePathStyle::Posix => "不能使用服务器根目录 /",
            RemotePathStyle::WindowsDrive => "不能使用 Windows 盘符根目录",
            RemotePathStyle::Unc => "UNC 路径必须位于共享目录的子目录中",
        };
        return Err(AppError {
            message: format!("线上发布目录不安全：{detail}"),
        });
    }

    let normalized = match style {
        RemotePathStyle::Unc => format!("{prefix}{}", segments.join("/")),
        _ => format!("{prefix}{}", segments.join("/")),
    };
    Ok((normalized, style))
}

fn remote_path_comparison_key(path: &str, style: RemotePathStyle) -> String {
    if matches!(style, RemotePathStyle::WindowsDrive | RemotePathStyle::Unc) {
        path.to_lowercase()
    } else {
        path.to_string()
    }
}

fn paths_overlap(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|remainder| remainder.starts_with('/'))
        || right
            .strip_prefix(left)
            .is_some_and(|remainder| remainder.starts_with('/'))
}

fn remote_path_leaf(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or("app")
}

pub fn release_slug(input: &str, fallback_slug: &str) -> String {
    fn slug_part(value: &str) -> String {
        let mut slug = String::with_capacity(value.len());
        let mut last_dash = false;

        for ch in value.chars() {
            if ch.is_alphanumeric() {
                slug.extend(ch.to_lowercase());
                last_dash = false;
            } else if !last_dash {
                slug.push('-');
                last_dash = true;
            }
        }

        slug.trim_matches('-').to_string()
    }

    let slug = slug_part(input);
    if slug.is_empty() || slug == "app" {
        let fallback = slug_part(fallback_slug);
        if fallback.is_empty() || fallback == "app" {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            input.hash(&mut hasher);
            format!("app-{:x}", hasher.finish())
        } else {
            fallback
        }
    } else {
        slug
    }
}

pub fn normalize_and_validate_project(
    mut project: Project,
    peers: &[Project],
) -> Result<Project, AppError> {
    if !(0..=20).contains(&project.keep_versions) {
        return Err(AppError {
            message: "保留版本数必须在 0 到 20 之间".to_string(),
        });
    }

    let (live_root, live_style) = normalize_absolute_remote_path(&project.live_root_path)?;
    let releases_root = derive_releases_root_path(&live_root)?;
    let live_key = remote_path_comparison_key(&live_root, live_style);
    let releases_key = remote_path_comparison_key(&releases_root, live_style);
    if live_key == releases_key {
        return Err(AppError {
            message: "线上发布目录不能与版本归档目录相同，请不要将 live 目录命名为 releases".to_string(),
        });
    }

    let slug = release_slug(remote_path_leaf(&live_root), &project.name);
    for peer in peers {
        if peer.id == project.id || peer.server_id != project.server_id {
            continue;
        }

        let (peer_live, peer_style) = normalize_absolute_remote_path(&peer.live_root_path)
            .map_err(|error| AppError {
                message: format!("项目“{}”的远端目录无效：{}", peer.name, error.message),
            })?;
        if peer_style != live_style {
            continue;
        }

        let peer_live_key = remote_path_comparison_key(&peer_live, peer_style);
        if paths_overlap(&live_key, &peer_live_key) {
            return Err(AppError {
                message: format!(
                    "线上发布目录与同一服务器上的项目“{}”相同或互相嵌套",
                    peer.name
                ),
            });
        }

        let peer_releases = derive_releases_root_path(&peer_live)?;
        let peer_releases_key = remote_path_comparison_key(&peer_releases, peer_style);
        let peer_slug = release_slug(remote_path_leaf(&peer_live), &peer.name);
        if releases_key == peer_releases_key && slug == peer_slug {
            return Err(AppError {
                message: format!(
                    "版本目录标识“{}”与同一服务器上的项目“{}”冲突，请调整线上目录名称",
                    slug, peer.name
                ),
            });
        }
    }

    project.live_root_path = live_root;
    project.releases_root_path = releases_root;
    Ok(project)
}

// ---------------------------------------------------------------------------
// AES-256-GCM 密码加密与解密
// ---------------------------------------------------------------------------

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore, Key, Nonce,
};

/// 与数据库存放在同一数据目录中的加密密钥文件路径。
const ENC_KEY_RELPATH: &str = ".encryption_key";

/// 计算加密密钥文件的绝对路径。
pub fn encryption_key_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.deploygo.app");
    data_dir.join(ENC_KEY_RELPATH)
}

/// 从磁盘加载 256 位密钥；首次调用时生成并持久化密钥。
fn get_or_create_key() -> Result<Key<Aes256Gcm>, AppError> {
    let path = encryption_key_path();
    if path.exists() {
        let hex = std::fs::read_to_string(&path)
            .map_err(|e| AppError { message: format!("无法读取加密密钥文件: {}", e) })?;
        let hex = hex.trim();
        let bytes = hex::decode(hex)
            .map_err(|e| AppError { message: format!("加密密钥文件格式错误: {}", e) })?;
        if bytes.len() != 32 {
            return Err(AppError { message: "加密密钥长度错误，期望 32 字节".to_string() });
        }
        let mut key = Key::<Aes256Gcm>::default();
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
    // 生成新密钥前，先尝试从旧数据目录迁移（旧版本使用 "deploygo"，而不是 bundle id 作为目录名）。
        let legacy_path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("deploygo")
            .join(ENC_KEY_RELPATH);
        if legacy_path.exists() {
    // 创建新的父目录并复制旧密钥。
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError { message: format!("无法创建数据目录: {}", e) })?;
            }
            std::fs::copy(&legacy_path, &path)
                .map_err(|e| AppError { message: format!("无法迁移加密密钥文件: {}", e) })?;
            let hex = std::fs::read_to_string(&path)
                .map_err(|e| AppError { message: format!("无法读取加密密钥文件: {}", e) })?;
            let hex = hex.trim();
            let bytes = hex::decode(hex)
                .map_err(|e| AppError { message: format!("加密密钥文件格式错误: {}", e) })?;
            if bytes.len() != 32 {
                return Err(AppError { message: "加密密钥长度错误，期望 32 字节".to_string() });
            }
            let mut key = Key::<Aes256Gcm>::default();
            key.copy_from_slice(&bytes);
            return Ok(key);
        }

        let key = Aes256Gcm::generate_key(OsRng);
        let hex = hex::encode(key);
    // 创建父目录。
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError { message: format!("无法创建数据目录: {}", e) })?;
        }
    // 以严格的文件权限写入。
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::write(&path, &hex)
                .and_then(|_| std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)))
                .map_err(|e| AppError { message: format!("无法写入加密密钥文件: {}", e) })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&path, &hex)
                .map_err(|e| AppError { message: format!("无法写入加密密钥文件: {}", e) })?;
        }
        Ok(key)
    }
}

/// 使用 AES-256-GCM 加密明文密码。
/// 返回 base64 字符串：`base64(12 字节 nonce || 密文 || 16 字节 tag)`。
pub fn encrypt_password(plaintext: &str) -> Result<String, AppError> {
    let key = get_or_create_key()?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 12 字节
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| AppError { message: format!("密码加密失败: {}", e) })?;
    // 拼接 nonce 和密文（密文已包含 tag）。
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    Ok(base64::engine::general_purpose::STANDARD_NO_PAD.encode(&combined))
}

/// 解密由 `encrypt_password` 生成的值。
/// 同时兼容新的 AES-GCM 格式和旧版 base64 明文（用于迁移），并返回明文密码。
pub fn decrypt_password(encrypted: &str) -> Result<String, AppError> {
    let key = get_or_create_key()?;
    decrypt_password_with_key_bytes(encrypted, key.as_slice())
}

pub fn decrypt_password_with_key_hex(
    encrypted: &str,
    key_hex: &str,
) -> Result<String, AppError> {
    let key = hex::decode(key_hex.trim()).map_err(|_| AppError {
        message: "备份中的旧加密密钥格式无效".to_string(),
    })?;
    if key.len() != 32 {
        return Err(AppError {
            message: "备份中的旧加密密钥长度无效".to_string(),
        });
    }
    decrypt_password_with_key_bytes(encrypted, &key)
}

fn decrypt_password_with_key_bytes(encrypted: &str, key: &[u8]) -> Result<String, AppError> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| AppError {
        message: "密码解密密钥长度无效".to_string(),
    })?;
    let combined = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(encrypted)
        .map_err(|_| AppError { message: "密码格式无效".to_string() })?;
    if combined.len() < 12 + 1 {
        return Err(AppError { message: "密码数据太短".to_string() });
    }
    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plain = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError { message: "密码解密失败，密钥可能已变更".to_string() })?;
    String::from_utf8(plain)
        .map_err(|_| AppError { message: "密码解密结果不是有效文本".to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, server_id: &str, live_root_path: &str) -> Project {
        Project {
            id: id.to_string(),
            name: id.to_string(),
            local_dist_path: "dist".to_string(),
            build_command: None,
            server_id: server_id.to_string(),
            keep_versions: 5,
            live_root_path: live_root_path.to_string(),
            releases_root_path: String::new(),
            sort_order: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_aes_gcm_roundtrip() {
        let plaintext = "my_s3cret_p@ss!";
        let encrypted = encrypt_password(plaintext).expect("encrypt should succeed");
        assert_ne!(encrypted, plaintext, "ciphertext should differ from plaintext");
        assert!(!encrypted.is_empty(), "ciphertext should not be empty");

        let decrypted = decrypt_password(&encrypted).expect("decrypt should succeed");
        assert_eq!(decrypted, plaintext, "roundtrip should match original");
    }

    #[test]
    fn test_aes_gcm_different_each_time() {
        let plaintext = "same_password";
        let e1 = encrypt_password(plaintext).expect("encrypt 1");
        let e2 = encrypt_password(plaintext).expect("encrypt 2");
    // nonce 随机生成，因此每次密文都应不同。
        assert_ne!(e1, e2, "each encryption should produce different output");
    }

    #[test]
    fn project_validation_rejects_dangerous_or_relative_roots() {
        for path in ["/", "relative/site", "/srv/site/../prod", "C:\\"] {
            assert!(
                normalize_and_validate_project(project("one", "server", path), &[]).is_err(),
                "{path}"
            );
        }
    }

    #[test]
    fn project_validation_normalizes_windows_remote_paths_without_host_os_rules() {
        let normalized = normalize_and_validate_project(
            project("one", "server", "C:\\inetpub\\wwwroot\\site\\"),
            &[],
        )
        .unwrap();

        assert_eq!(normalized.live_root_path, "C:/inetpub/wwwroot/site");
        assert_eq!(normalized.releases_root_path, "C:/inetpub/wwwroot/releases");
    }

    #[test]
    fn project_validation_rejects_release_root_and_nested_live_roots() {
        assert!(normalize_and_validate_project(
            project("one", "server", "/srv/releases"),
            &[],
        )
        .is_err());

        let existing = project("existing", "server", "/srv/site");
        assert!(normalize_and_validate_project(
            project("new", "server", "/srv/site/admin"),
            &[existing],
        )
        .is_err());
    }

    #[test]
    fn project_validation_rejects_release_slug_collisions() {
        let existing = project("existing", "server", "/srv/foo_bar");
        assert!(normalize_and_validate_project(
            project("new", "server", "/srv/foo-bar"),
            &[existing],
        )
        .is_err());
    }

    #[test]
    fn app_slug_uses_a_stable_hash_instead_of_recursing() {
        let slug = release_slug("app", "app");
        assert!(slug.starts_with("app-"));
    }

}
