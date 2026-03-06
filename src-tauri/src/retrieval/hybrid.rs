use std::collections::HashSet;

use crate::{retrieval::models::RetrievedChunk, storage::keyword_store::KeywordStore};

pub async fn merge_with_keyword(
    vector_chunks: Vec<RetrievedChunk>,
    keyword: &KeywordStore,
    question: &str,
    k: usize,
) -> anyhow::Result<Vec<RetrievedChunk>> {
    let keyword_ids = keyword.query(question, k).await?;
    if keyword_ids.is_empty() {
        return Ok(vector_chunks);
    }

    let mut seen: HashSet<String> = vector_chunks.iter().map(|c| c.chunk_id.clone()).collect();
    let mut out = vector_chunks;

    for id in keyword_ids {
        if seen.contains(&id) {
            continue;
        }
        seen.insert(id.clone());
        out.push(RetrievedChunk {
            chunk_id: id,
            file: "unknown".to_string(),
            text: String::new(),
            score: 0.01,
            page_start: None,
            page_end: None,
        });
    }

    out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    out.truncate(k);
    Ok(out)
}
