mod commands;
mod error;
mod persistence;
mod secrets;

use rcon_client::ConnectionRegistry;
use tauri::Manager;

use crate::persistence::Db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // App-data dir is OS-specific (Application Support on macOS,
            // %APPDATA% on Windows). We nest under `cesm/` so users can
            // wipe local state by deleting one folder.
            let data_dir = app.path().app_data_dir()?;
            let db_path = data_dir.join("cesm").join("db.sqlite");
            tracing::info!(path = %db_path.display(), "opening database");

            let db = Db::open(&db_path).map_err(|e| {
                tracing::error!(error = ?e, "failed to open database");
                Box::<dyn std::error::Error>::from(format!("{:?}", e))
            })?;
            app.manage(db);
            app.manage(ConnectionRegistry::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::servers::list_servers,
            commands::servers::save_server,
            commands::servers::delete_server,
            commands::servers::test_connection,
            commands::servers::open_connection,
            commands::servers::close_connection,
            commands::servers::connection_state,
            commands::servers::send_command,
            commands::settings::get_setting,
            commands::settings::set_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
