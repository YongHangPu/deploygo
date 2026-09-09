//! 部署引擎接入层。
//!
//! 开源构建默认使用 `stub`（不含引擎）；官方完整版通过 deploygo-core 的
//! `scripts/apply-overlay.mjs` 将私有引擎注入 `real/` 目录（已被 .gitignore
//! 忽略，不会提交），build.rs 检测到该目录后启用 `deploygo_engine` cfg，
//! 自动切换为真实引擎。

pub mod contract;

#[cfg(deploygo_engine)]
mod real;

#[cfg(not(deploygo_engine))]
mod stub;

#[cfg(deploygo_engine)]
pub use real::Backend;

#[cfg(not(deploygo_engine))]
pub use stub::Backend;
