use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{app_state::AppState, errors::AppError, retrieval::models::Citation};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskRequest {
    pub question: String,
    pub top_k: Option<usize>,
    pub hybrid: Option<bool>,
    pub rerank: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AskResponse {
    pub answer: String,
    pub not_found: bool,
    pub citations: Vec<Citation>,
    pub latency_ms: i64,
}

#[tauri::command]
pub async fn ask_question(
    state: State<'_, Arc<AppState>>,
    request: AskRequest,
) -> Result<AskResponse, AppError> {
    let response = state.query_engine.ask(request).await?;
    Ok(response)
}
