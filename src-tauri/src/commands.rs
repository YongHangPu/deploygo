use std::sync::Arc;

use tauri::{AppHandle, State};

use crate::backend::contract::LicenseInfo;
use crate::backup::{self, CredentialData, ExportData};
use crate::config::{
    self, decrypt_password, decrypt_password_with_key_hex, encrypt_password,
    normalize_and_validate_project, Project, ServerConfig,
};
use crate::db::{self, Database};
use crate::error::AppError;

// ---- 状态容器 ----

pub struct AppState {
    pub config_store: config::ConfigStore,
    pub db: Arc<Database>,
    pub backend: Arc<crate::backend::Backend>,
}

// ---- Tauri 命令 ----

#[tauri::command]
pub fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, AppError> {
    state.db.get_projects()
}

#[tauri::command]
pub fn get_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<Option<Project>, AppError> {
    state.db.get_project(&project_id)
}

#[tauri::command]
pub fn save_project(state: State<'_, AppState>, project: Project) -> Result<(), AppError> {
    let peers = state.db.get_projects()?;
    let project = normalize_and_validate_project(project, &peers)?;
    state.db.save_project(&project)
}

#[tauri::command]
pub fn delete_project(state: State<'_, AppState>, project_id: String) -> Result<(), AppError> {
    state.db.delete_project(&project_id)
}

#[tauri::command]
pub fn get_servers(state: State<'_, AppState>) -> Result<Vec<ServerConfig>, AppError> {
    state.db.get_servers()
}

#[tauri::command]
pub fn get_server(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<Option<ServerConfig>, AppError> {
    state.db.get_server(&server_id)
}

#[tauri::command]
pub fn save_server(
    state: State<'_, AppState>,
    server: ServerConfig,
    password: Option<String>,
    jump_password: Option<String>,
) -> Result<(), AppError> {
    if let Some(pwd) = password {
        if pwd.is_empty() {
            state.db.delete_password_tagged(&server.id, "main")?;
        } else {
            let encoded = encrypt_password(&pwd)?;
            state.db.store_password(&server.id, &encoded)?;
        }
    }
    if server.jump_host.is_none() {
        state.db.delete_password_tagged(&server.id, "jump")?;
    } else if let Some(pwd) = jump_password {
        if pwd.is_empty() {
            state.db.delete_password_tagged(&server.id, "jump")?;
        } else {
            let encoded = encrypt_password(&pwd)?;
            state
                .db
                .store_password_tagged(&server.id, &encoded, "jump")?;
        }
    }
    state.db.save_server(&server)
}

#[tauri::command]
pub fn delete_server(state: State<'_, AppState>, server_id: String) -> Result<(), AppError> {
    state.db.delete_server(&server_id)
}

#[tauri::command]
pub fn reorder_projects(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), AppError> {
    state.db.reorder_projects(&ids)
}

#[tauri::command]
pub fn reorder_servers(state: State<'_, AppState>, ids: Vec<String>) -> Result<(), AppError> {
    state.db.reorder_servers(&ids)
}

#[tauri::command]
pub async fn test_connection(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    server_id: String,
    password: Option<String>,
) -> Result<u64, AppError> {
    let backend = state.backend.clone();
    backend.test_connection(app_handle, server_id, password).await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn test_connection_direct(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    host: String,
    port: u16,
    username: String,
    password: String,
    auth_method: Option<String>,
    key_path: Option<String>,
    jump_host: Option<String>,
    jump_port: Option<u16>,
    jump_username: Option<String>,
    jump_password: Option<String>,
    jump_auth_method: Option<String>,
    jump_key_path: Option<String>,
    server_id: Option<String>,
) -> Result<u64, AppError> {
    let backend = state.backend.clone();
    backend
        .test_connection_direct(
            app_handle,
            host,
            port,
            username,
            password,
            auth_method,
            key_path,
            jump_host,
            jump_port,
            jump_username,
            jump_password,
            jump_auth_method,
            jump_key_path,
            server_id,
        )
        .await
}

#[tauri::command]
pub fn get_server_password(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<bool, AppError> {
    let pwd = state.db.get_password(&server_id)?;
    Ok(pwd.is_some())
}

#[tauri::command]
pub fn get_jump_password_exists(
    state: State<'_, AppState>,
    server_id: String,
) -> Result<bool, AppError> {
    let pwd = state.db.get_password_tagged(&server_id, "jump")?;
    Ok(pwd.is_some())
}

#[tauri::command]
pub async fn start_deploy(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    project_id: String,
    password: Option<String>,
) -> Result<i64, AppError> {
    let backend = state.backend.clone();
    backend.start_deploy(app_handle, project_id, password).await
}

#[tauri::command]
pub fn cancel_deploy(
    state: State<'_, AppState>,
) -> Result<crate::backend::contract::CancelDeployResult, AppError> {
    Ok(state.backend.cancel_deploy())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn get_deploy_history(
    state: State<'_, AppState>,
    project_id: Option<String>,
    server_id: Option<String>,
    status: Option<String>,
    date_from: Option<String>,
    date_to: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<db::PaginatedResult<crate::config::DeployRecord>, AppError> {
    let filter = db::HistoryFilter {
        project_id,
        server_id,
        status,
        date_from,
        date_to,
        limit,
        offset,
    };
    state.db.get_history(&filter)
}

#[tauri::command]
pub fn get_deploy_record(
    state: State<'_, AppState>,
    record_id: i64,
) -> Result<Option<crate::config::DeployRecord>, AppError> {
    state.db.get_record(record_id)
}

#[tauri::command]
pub fn delete_deploy_record(state: State<'_, AppState>, record_id: i64) -> Result<(), AppError> {
    state.db.delete_record(record_id)
}

#[tauri::command]
pub fn export_log(state: State<'_, AppState>, record_id: i64) -> Result<String, AppError> {
    let record = state.db.get_record(record_id)?.ok_or_else(|| AppError {
        message: "部署记录不存在".to_string(),
    })?;

    let data_dir = state.config_store.get_config().data_dir.clone();
    let export_path = data_dir.join(format!("deploy_log_{}.txt", record_id));
    std::fs::write(&export_path, &record.log)?;

    Ok(export_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn rollback_to_version(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    record_id: i64,
    password: Option<String>,
) -> Result<i64, AppError> {
    let backend = state.backend.clone();
    backend.rollback_to_version(app_handle, record_id, password).await
}

#[tauri::command]
pub fn save_project_with_id(state: State<'_, AppState>, project: Project) -> Result<(), AppError> {
    let peers = state.db.get_projects()?;
    let project = normalize_and_validate_project(project, &peers)?;
    state.db.save_project(&project)
}

#[tauri::command]
pub fn get_app_data_dir(state: State<'_, AppState>) -> Result<String, AppError> {
    let dir = state
        .config_store
        .get_config()
        .data_dir
        .to_string_lossy()
        .to_string();
    Ok(dir)
}

#[tauri::command]
pub fn validate_build_dir(path: String) -> Result<String, String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("构建产物目录不存在: {}", path));
    }
    if !p.is_dir() {
        return Err(format!("路径不是目录: {}", path));
    }
    if p.join("index.html").is_file() {
        return Ok("ok".into());
    }
    if let Ok(entries) = std::fs::read_dir(p) {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && entry.path().join("index.html").is_file()
            {
                return Ok("ok".into());
            }
        }
    }
    Err(format!(
        "构建产物目录中未找到 index.html: {}\n请确认已执行构建命令",
        path
    ))
}

#[tauri::command]
pub fn export_config(
    state: State<'_, AppState>,
    include_credentials: Option<bool>,
    backup_password: Option<String>,
) -> Result<ExportData, AppError> {
    let servers = state.db.get_servers()?;
    let projects = state.db.get_projects()?;

    let credential_payload = if include_credentials.unwrap_or(false) {
        let password = backup_password.as_deref().ok_or_else(|| AppError {
            message: "包含凭据的备份必须设置备份密码".to_string(),
        })?;
        let credentials = collect_plaintext_credentials(&state.db, &servers)?;
        Some(backup::encrypt_credentials(&credentials, password)?)
    } else {
        None
    };

    Ok(ExportData::v3(servers, projects, credential_payload))
}

#[tauri::command]
pub fn import_config(
    state: State<'_, AppState>,
    mut data: ExportData,
    overwrite: bool,
    backup_password: Option<String>,
) -> Result<String, AppError> {
    let credentials = decode_import_credentials(&data, backup_password.as_deref())?;

    let existing_projects = state.db.get_projects()?;
    let existing_ids = existing_projects
        .iter()
        .map(|project| project.id.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut effective_projects = existing_projects;
    for project in &data.projects {
        if let Some(index) = effective_projects
            .iter()
            .position(|existing| existing.id == project.id)
        {
            if overwrite {
                effective_projects[index] = project.clone();
            }
        } else {
            effective_projects.push(project.clone());
        }
    }

    let normalized_projects = effective_projects
        .iter()
        .cloned()
        .map(|project| normalize_and_validate_project(project, &effective_projects))
        .collect::<Result<Vec<_>, AppError>>()?;
    let normalized_by_id = normalized_projects
        .into_iter()
        .map(|project| (project.id.clone(), project))
        .collect::<std::collections::HashMap<_, _>>();
    data.projects = data
        .projects
        .into_iter()
        .map(|project| {
            if !overwrite && existing_ids.contains(&project.id) {
                project
            } else {
                normalized_by_id
                    .get(&project.id)
                    .cloned()
                    .unwrap_or(project)
            }
        })
        .collect();

    // 在开启数据库事务前先完成全部加密，避免加密失败时留下不完整的导入配置。
    let passwords = credentials
        .passwords
        .iter()
        .map(|(server_id, plaintext)| Ok((server_id.clone(), encrypt_password(plaintext)?)))
        .collect::<Result<std::collections::HashMap<_, _>, AppError>>()?;
    let jump_passwords = credentials
        .jump_passwords
        .iter()
        .map(|(server_id, plaintext)| Ok((server_id.clone(), encrypt_password(plaintext)?)))
        .collect::<Result<std::collections::HashMap<_, _>, AppError>>()?;

    let stats = state.db.import_configuration(
        &data.servers,
        &data.projects,
        &passwords,
        &jump_passwords,
        overwrite,
    )?;

    let summary = format!(
        "导入完成：服务器 {} 个（跳过 {} 个），项目 {} 个（跳过 {} 个），密码 {} 个",
        stats.imported_servers,
        stats.skipped_servers,
        stats.imported_projects,
        stats.skipped_projects,
        stats.imported_passwords,
    );
    Ok(summary)
}

fn collect_plaintext_credentials(
    db: &Database,
    servers: &[ServerConfig],
) -> Result<CredentialData, AppError> {
    let mut credentials = CredentialData::default();
    for server in servers {
        if let Some(encrypted) = db.get_password(&server.id)? {
            credentials
                .passwords
                .insert(server.id.clone(), decrypt_password(&encrypted)?);
        }
        if let Some(encrypted) = db.get_password_tagged(&server.id, "jump")? {
            credentials
                .jump_passwords
                .insert(server.id.clone(), decrypt_password(&encrypted)?);
        }
    }
    Ok(credentials)
}

fn decode_import_credentials(
    data: &ExportData,
    backup_password: Option<&str>,
) -> Result<CredentialData, AppError> {
    match data.version {
        3 => match &data.credential_payload {
            Some(payload) => {
                let password = backup_password.ok_or_else(|| AppError {
                    message: "该备份包含加密凭据，请输入备份密码".to_string(),
                })?;
                backup::decrypt_credentials(payload, password)
            }
            None => Ok(CredentialData::default()),
        },
        2 => {
            let decrypt = |encrypted: &str| match data.encryption_key.as_deref() {
                Some(key) => decrypt_password_with_key_hex(encrypted, key),
                None => decrypt_password(encrypted),
            };
            let mut credentials = CredentialData::default();
            for (server_id, encrypted) in &data.passwords {
                credentials
                    .passwords
                    .insert(server_id.clone(), decrypt(encrypted)?);
            }
            for (server_id, encrypted) in &data.jump_passwords {
                credentials
                    .jump_passwords
                    .insert(server_id.clone(), decrypt(encrypted)?);
            }
            Ok(credentials)
        }
        version => Err(AppError {
            message: format!("不支持的配置备份版本: {}", version),
        }),
    }
}

// ─── 许可证命令 ───────────────────────────────────────────────────────────────

/// 使用许可证密钥激活。验证签名并按激活时间计算有效期后写入数据库。
#[tauri::command]
pub fn activate_license(state: State<'_, AppState>, key: String) -> Result<LicenseInfo, AppError> {
    state.backend.activate_license(&key)
}

/// 获取当前许可证状态。每次调用都会重新验证已保存的密钥。
/// 覆盖试用中、正式许可证有效和已过期三种状态。
#[tauri::command]
pub fn get_license_status(state: State<'_, AppState>) -> Result<LicenseInfo, AppError> {
    state.backend.get_license_status()
}

/// 判断应用是否运行在开发模式（`pnpm tauri:dev`）。
/// 只有开发模式才显示密钥生成器界面。
#[tauri::command]
pub fn is_dev_mode(state: State<'_, AppState>) -> bool {
    state.backend.is_dev_mode()
}

#[tauri::command]
pub fn is_updater_configured() -> bool {
    option_env!("DEPLOYGO_UPDATER_ENDPOINT").is_some_and(|value| !value.trim().is_empty())
        && option_env!("DEPLOYGO_UPDATER_PUBLIC_KEY").is_some_and(|value| !value.trim().is_empty())
}

/// 生成新的 Ed25519 密钥对，仅限开发模式。
#[cfg(debug_assertions)]
#[tauri::command]
pub fn generate_license_keypair(state: State<'_, AppState>) -> Result<(String, String), String> {
    state.backend.generate_license_keypair()
}

/// 签发许可证密钥，仅限开发模式。
#[cfg(debug_assertions)]
#[tauri::command]
pub fn sign_license_key(
    state: State<'_, AppState>,
    tier: String,
    duration_value: u32,
    duration_unit: String,
    secret_hex: String,
) -> Result<String, String> {
    state
        .backend
        .sign_license_key(&tier, duration_value, &duration_unit, &secret_hex)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
    use base64::Engine;

    use super::*;

    #[test]
    fn v2_credentials_are_decrypted_with_the_embedded_source_key() {
        let key = [7u8; 32];
        let nonce = [3u8; 12];
        let cipher = Aes256Gcm::new_from_slice(&key).unwrap();
        let ciphertext = cipher
            .encrypt(Nonce::from_slice(&nonce), b"legacy-secret".as_slice())
            .unwrap();
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);
        let encrypted = base64::engine::general_purpose::STANDARD_NO_PAD.encode(combined);

        let data = ExportData {
            version: 2,
            exported_at: String::new(),
            servers: Vec::new(),
            projects: Vec::new(),
            credential_mode: None,
            credential_payload: None,
            passwords: HashMap::from([("server-1".to_string(), encrypted)]),
            jump_passwords: HashMap::new(),
            encryption_key: Some(hex::encode(key)),
        };

        let credentials = decode_import_credentials(&data, None).unwrap();
        assert_eq!(credentials.passwords["server-1"], "legacy-secret");
    }
}
