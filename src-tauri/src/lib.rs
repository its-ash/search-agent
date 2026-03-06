pub mod app_state;
pub mod commands;
pub mod config;
pub mod errors;
pub mod ingestion;
pub mod jobs;
pub mod retrieval;
pub mod security;
pub mod server;
pub mod storage;
pub mod telemetry;

use std::sync::Arc;

use app_state::AppState;
use tauri::{Manager, RunEvent};
use tracing::error;

pub fn run() {
    telemetry::logging::init_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let state = Arc::new(AppState::bootstrap(&handle).await?);
                if let Err(err) = state.server_manager.initialize().await {
                    error!("llama server initialization failed: {err}");
                }
                handle.manage(state);
                Ok::<(), anyhow::Error>(())
            })?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan::start_scan,
            commands::scan::get_scan_status,
            commands::scan::cancel_scan,
            commands::query::ask_question,
            commands::server::get_server_status,
            commands::server::restart_server,
            commands::index_admin::rebuild_index,
            commands::index_admin::reset_index,
            commands::index_admin::get_index_stats
        ])
        .build(tauri::generate_context!())
        .expect("failed to build tauri app")
        .run(|app_handle, event| {
            if let RunEvent::ExitRequested { .. } = event {
                let state = app_handle.try_state::<Arc<AppState>>().map(|s| s.inner().clone());
                if let Some(state) = state {
                    tauri::async_runtime::block_on(async move {
                        let _ = state.server_manager.shutdown_if_owned().await;
                    });
                }
            }
        });
}
