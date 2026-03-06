use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatus {
    pub job_id: String,
    pub status: String,
    pub total_files: i64,
    pub processed_files: i64,
    pub failed_files: i64,
    pub current_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureEntry {
    pub path: String,
    pub stage: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone)]
pub struct DocumentRecord {
    pub id: String,
    pub abs_path: String,
    pub rel_path: String,
    pub file_ext: String,
    pub file_hash: String,
    pub mtime_ms: i64,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPayload {
    pub chunk_id: String,
    pub document_id: String,
    pub abs_path: String,
    pub rel_path: String,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub section: Option<String>,
    pub text: String,
    pub embedding: Vec<f32>,
}
