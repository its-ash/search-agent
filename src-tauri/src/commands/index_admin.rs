use std::sync::Arc;

use serde::Deserialize;
use serde::Serialize;
use tauri::State;

use crate::{app_state::AppState, errors::AppError};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildRequest {
    pub root_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatsResponse {
    pub indexed_files: i64,
    pub indexed_chunks: i64,
}

#[tauri::command]
pub async fn rebuild_index(
    state: State<'_, Arc<AppState>>,
    request: RebuildRequest,
) -> Result<serde_json::Value, AppError> {
    if let Some(path) = request.root_path {
        state.ingestion.rebuild(path).await?;
    } else {
        state.ingestion.rebuild_last().await?;
    }
    Ok(serde_json::json!({ "jobId": state.ingestion.latest_job_id().await }))
}

#[tauri::command]
pub async fn reset_index(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, AppError> {
    state.ingestion.reset().await?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn get_index_stats(state: State<'_, Arc<AppState>>) -> Result<IndexStatsResponse, AppError> {
    let (indexed_files, indexed_chunks) = state.sqlite.get_index_counts().await?;
    Ok(IndexStatsResponse {
        indexed_files,
        indexed_chunks,
    })
}
