//! 构建脚本：检测 overlay 注入的私有引擎目录，切换完整版构建。
//!
//! `src/backend/real/` 目录存在时（由 deploygo-core 的
//! `scripts/apply-overlay.mjs on` 注入），启用 `deploygo_engine` cfg，
//! 使 `backend` 模块使用真实引擎；目录不存在时使用开源桩实现。

fn main() {
    // Tauri 官方构建脚本：嵌入 Windows manifest（Common-Controls 6.0 依赖、DPI 声明）、
    // exe 图标等资源。缺失会导致 Windows 上启动即报
    // 「无法定位程序输入点 TaskDialogIndirect」。
    tauri_build::build();

    // real/ 目录的增删本身会改变 backend/ 目录内容，此处声明监视路径，
    // 确保 overlay on/off 后重新编译。若切换未生效，可执行 cargo clean。
    println!("cargo:rerun-if-changed=src/backend/real");
    println!("cargo:rerun-if-changed=src/backend/stub.rs");
    println!("cargo:rustc-check-cfg=cfg(deploygo_engine)");
    // tauri 模板在 lib.rs 使用 #[cfg_attr(mobile, ...)]，声明该 cfg 消除警告。
    println!("cargo:rustc-check-cfg=cfg(mobile)");

    if std::path::Path::new("src/backend/real").exists() {
        println!("cargo:rustc-cfg=deploygo_engine");
    }
}
