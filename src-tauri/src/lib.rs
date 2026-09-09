mod backend;
mod backup;
mod commands;
mod config;
mod db;
mod error;

use std::sync::Arc;

use commands::AppState;

macro_rules! deploygo_handlers {
    ($($debug_command:path),* $(,)?) => {
        tauri::generate_handler![
            commands::get_projects,
            commands::get_project,
            commands::save_project,
            commands::delete_project,
            commands::get_servers,
            commands::get_server,
            commands::save_server,
            commands::delete_server,
            commands::reorder_projects,
            commands::reorder_servers,
            commands::test_connection,
            commands::test_connection_direct,
            commands::get_server_password,
            commands::get_jump_password_exists,
            commands::start_deploy,
            commands::cancel_deploy,
            commands::get_deploy_history,
            commands::get_deploy_record,
            commands::delete_deploy_record,
            commands::export_log,
            commands::rollback_to_version,
            commands::save_project_with_id,
            commands::get_app_data_dir,
            commands::validate_build_dir,
            commands::export_config,
            commands::import_config,
            commands::activate_license,
            commands::get_license_status,
            commands::is_dev_mode,
            commands::is_updater_configured,
            $($debug_command),*
        ]
    };
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化配置（数据目录等）。
    let config_store = config::ConfigStore::new().expect("Failed to initialize config store");
    let db_path = config_store.get_config().db_path.clone();

    // 初始化数据库（创建数据表）。
    let database = Arc::new(db::Database::new(&db_path).expect("Failed to initialize database"));

    // 初始化后端：开源纯壳构建为桩实现；overlay 注入 real/ 后为完整引擎。
    let backend =
        Arc::new(backend::Backend::new(database.clone(), &db_path).expect("Failed to initialize backend"));
    backend
        .init()
        .expect("Failed to initialize license state");

    let app_state = AppState {
        config_store,
        db: database,
        backend,
    };

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(app_state);

    #[cfg(debug_assertions)]
    let builder = builder.invoke_handler(deploygo_handlers![
        commands::generate_license_keypair,
        commands::sign_license_key,
    ]);
    #[cfg(not(debug_assertions))]
    let builder = builder.invoke_handler(deploygo_handlers![]);

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
