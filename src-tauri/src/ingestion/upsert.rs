use crate::storage::{models::ChunkPayload, sqlite::SqliteStore, vector_store::VectorStore};

pub async fn upsert_chunks(
    sqlite: &SqliteStore,
    vector: &VectorStore,
    document_id: &str,
    chunks: Vec<ChunkPayload>,
) -> anyhow::Result<()> {
    let ids: Vec<String> = chunks.iter().map(|c| c.chunk_id.clone()).collect();
    vector.upsert_chunks(&chunks).await?;
    sqlite.save_chunk_refs(document_id, &ids).await?;
    Ok(())
}
