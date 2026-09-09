//! 开源纯壳构建的 Backend 桩实现。
//!
//! 开源仓库不包含部署引擎（SSH 上传、原子发布、回滚）与授权校验逻辑，
//! 本桩保证应用可以编译、启动与浏览界面，但部署相关操作会返回引导信息，
//! 提示用户下载包含完整引擎的官方版本。
//!
//! 完整版引擎位于私有仓库 deploygo-core，经 overlay 注入 `real/` 目录后
//! 由 build.rs 启用 `deploygo_engine` cfg，切换为 `real::Backend`。

use std::path::Path;
use std::sync::Arc;

use tauri::AppHandle;

use crate::backend::contract::{CancelDeployResult, LicenseInfo, ENGINE_SHELL};
use crate::db::Database;
use crate::error::AppError;

const ENGINE_MISSING: &str = "此构建未包含部署引擎。开源仓库仅提供界面与方案展示，\
请从 GitHub Releases（https://github.com/YongHangPu/deploygo/releases）下载官方完整版。";

fn shell_status() -> LicenseInfo {
    LicenseInfo {
        license_id: String::new(),
        tier: ENGINE_SHELL.to_string(),
        expires_at: String::new(),
        is_valid: false,
        reason: Some(ENGINE_MISSING.to_string()),
        trial_remaining_days: 0,
        can_deploy: false,
        engine: ENGINE_SHELL.to_string(),
    }
}

pub struct Backend;

impl Backend {
    pub fn new(_db: Arc<Database>, _db_path: &Path) -> Result<Self, AppError> {
        Ok(Backend)
    }

    /// 纯壳构建无需初始化授权状态。
    pub fn init(&self) -> Result<(), AppError> {
        Ok(())
    }

    pub fn is_dev_mode(&self) -> bool {
        cfg!(debug_assertions)
    }

    pub async fn test_connection(
        &self,
        _app_handle: AppHandle,
        _server_id: String,
        _password: Option<String>,
    ) -> Result<u64, AppError> {
        Err(engine_missing())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn test_connection_direct(
        &self,
        _app_handle: AppHandle,
        _host: String,
        _port: u16,
        _username: String,
        _password: String,
        _auth_method: Option<String>,
        _key_path: Option<String>,
        _jump_host: Option<String>,
        _jump_port: Option<u16>,
        _jump_username: Option<String>,
        _jump_password: Option<String>,
        _jump_auth_method: Option<String>,
        _jump_key_path: Option<String>,
        _server_id: Option<String>,
    ) -> Result<u64, AppError> {
        Err(engine_missing())
    }

    pub async fn start_deploy(
        &self,
        _app_handle: AppHandle,
        _project_id: String,
        _password: Option<String>,
    ) -> Result<i64, AppError> {
        Err(engine_missing())
    }

    pub fn cancel_deploy(&self) -> CancelDeployResult {
        CancelDeployResult::NotRunning
    }

    pub async fn rollback_to_version(
        &self,
        _app_handle: AppHandle,
        _record_id: i64,
        _password: Option<String>,
    ) -> Result<i64, AppError> {
        Err(engine_missing())
    }

    pub fn activate_license(&self, _key: &str) -> Result<LicenseInfo, AppError> {
        Ok(shell_status())
    }

    pub fn get_license_status(&self) -> Result<LicenseInfo, AppError> {
        Ok(shell_status())
    }

    /// 密钥生成器仅存在于完整版的开发模式构建中。
    #[cfg(debug_assertions)]
    pub fn generate_license_keypair(&self) -> Result<(String, String), String> {
        Err(ENGINE_MISSING.to_string())
    }

    #[cfg(debug_assertions)]
    pub fn sign_license_key(
        &self,
        _tier: &str,
        _duration_value: u32,
        _duration_unit: &str,
        _secret_hex: &str,
    ) -> Result<String, String> {
        Err(ENGINE_MISSING.to_string())
    }
}

fn engine_missing() -> AppError {
    AppError {
        message: ENGINE_MISSING.to_string(),
    }
}
