use crate::{ingestion::embeddings, server::lifecycle::ServerManager};

pub async fn embed_query(server: &ServerManager, q: &str) -> anyhow::Result<Vec<f32>> {
    let model = embeddings::default_embedding_model();
    let vectors = embeddings::embed_batch(server, &[q.to_string()], &model).await?;
    vectors
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing embedding"))
}
