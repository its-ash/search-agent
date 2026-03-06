use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{app_state::AppState, errors::AppError, storage::models::FailureEntry};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanRequest {
    pub root_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartScanResponse {
    pub job_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusRequest {
    pub job_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatusResponse {
    pub job_id: String,
    pub status: String,
    pub total_files: i64,
    pub processed_files: i64,
    pub failed_files: i64,
    pub current_file: Option<String>,
    pub failures: Vec<FailureEntry>,
}

#[tauri::command]
pub async fn start_scan(
    state: State<'_, Arc<AppState>>,
    request: StartScanRequest,
) -> Result<StartScanResponse, AppError> {
    let job_id = state.ingestion.start_scan(request.root_path).await?;
    Ok(StartScanResponse { job_id })
}

#[tauri::command]
pub async fn cancel_scan(
    state: State<'_, Arc<AppState>>,
    request: ScanStatusRequest,
) -> Result<serde_json::Value, AppError> {
    state.ingestion.cancel_scan(&request.job_id).await?;
    Ok(serde_json::json!({ "ok": true }))
}

#[tauri::command]
pub async fn get_scan_status(
    state: State<'_, Arc<AppState>>,
    request: ScanStatusRequest,
) -> Result<ScanStatusResponse, AppError> {
    let status = state.sqlite.get_scan_status(&request.job_id).await?;
    let failures = state.sqlite.get_failures(&request.job_id).await?;
    Ok(ScanStatusResponse {
        job_id: status.job_id,
        status: status.status,
        total_files: status.total_files,
        processed_files: status.processed_files,
        failed_files: status.failed_files,
        current_file: status.current_file,
        failures,
    })
}
