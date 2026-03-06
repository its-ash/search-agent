use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::{app_state::AppState, errors::AppError};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatusResponse {
    pub status: String,
    pub ownership: String,
    pub endpoint: String,
    pub pid: Option<u32>,
    pub message: Option<String>,
}

#[tauri::command]
pub async fn get_server_status(
    state: State<'_, Arc<AppState>>,
) -> Result<ServerStatusResponse, AppError> {
    let s = state.server_manager.status_snapshot().await;
    Ok(ServerStatusResponse {
        status: s.status,
        ownership: s.ownership,
        endpoint: s.endpoint,
        pid: s.pid,
        message: s.message,
    })
}

#[tauri::command]
pub async fn restart_server(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, AppError> {
    state.server_manager.restart().await?;
    Ok(serde_json::json!({ "ok": true }))
}
