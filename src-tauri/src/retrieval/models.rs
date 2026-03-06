use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub file: String,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub chunk_id: String,
    pub excerpt: String,
}

#[derive(Debug, Clone)]
pub struct RetrievedChunk {
    pub chunk_id: String,
    pub file: String,
    pub text: String,
    pub score: f32,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
}
