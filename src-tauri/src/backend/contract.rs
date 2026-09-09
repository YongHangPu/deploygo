//! 前后端契约类型：真实引擎（完整版）与桩实现（开源纯壳）共用。
//!
//! 开源仓库默认编译为纯壳（stub），不含部署引擎；
//! 官方完整版通过 deploygo-core 的 overlay 脚本注入 `real/` 目录后，
//! 由 build.rs 启用 `deploygo_engine` cfg 切换为真实引擎。

use serde::{Deserialize, Serialize};

/// 许可证状态信息，由 `get_license_status` / `activate_license` 返回。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub license_id: String,
    pub tier: String,
    pub expires_at: String,
    /// 当前许可证是否有效（试用中为 false，但 can_deploy 为 true）。
    pub is_valid: bool,
    /// `is_valid` 为 false 时显示给用户的原因。
    pub reason: Option<String>,
    /// 剩余试用天数（小于 0 表示试用结束，0 表示最后一天）。
    pub trial_remaining_days: i32,
    /// 当前许可证是否允许部署。
    pub can_deploy: bool,
    /// 构建类型标识：`"full"`（含部署引擎）或 `"shell"`（开源纯壳）。
    pub engine: String,
}

/// 取消部署请求的结果。
/// `Accepted` / `TooLate` 仅由完整版引擎（overlay 注入）构造。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelDeployResult {
    /// 已受理取消请求。
    Accepted,
    /// 已进入提交阶段，取消为时已晚。
    TooLate,
    /// 当前没有正在进行的部署任务。
    NotRunning,
}

/// 开源纯壳构建的引擎标识。仅由桩实现（stub.rs）使用。
#[allow(dead_code)]
pub const ENGINE_SHELL: &str = "shell";
