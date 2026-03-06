use crate::{retrieval::models::RetrievedChunk, storage::vector_store::VectorStore};

pub async fn top_k(vector: &VectorStore, emb: &[f32], k: usize) -> anyhow::Result<Vec<RetrievedChunk>> {
    let results = vector.search(emb, k).await?;
    Ok(results
        .into_iter()
        .map(|(chunk, score)| RetrievedChunk {
            chunk_id: chunk.chunk_id,
            file: chunk.rel_path,
            text: chunk.text,
            score,
            page_start: chunk.page_start,
            page_end: chunk.page_end,
        })
        .collect())
}
