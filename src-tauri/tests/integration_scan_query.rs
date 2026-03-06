use std::sync::Arc;

use search_agent_lib::{
    retrieval::vector_search,
    storage::{models::ChunkPayload, vector_store::VectorStore},
};
use tempfile::tempdir;

#[tokio::test]
async fn scan_index_query_happy_path_like() {
    let dir = tempdir().expect("temp dir");
    let vector = Arc::new(VectorStore::new(dir.path()).expect("vector store"));

    let chunk = ChunkPayload {
        chunk_id: "doc1:0".into(),
        document_id: "doc1".into(),
        abs_path: "/tmp/doc1.txt".into(),
        rel_path: "doc1.txt".into(),
        page_start: Some(1),
        page_end: Some(1),
        section: Some("intro".into()),
        text: "search agent supports local first retrieval".into(),
        embedding: vec![1.0, 0.0, 0.0],
    };

    vector.upsert_chunks(&[chunk]).await.expect("upsert");
    let results = vector_search::top_k(&vector, &[1.0, 0.0, 0.0], 3)
        .await
        .expect("search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].file, "doc1.txt");
}
