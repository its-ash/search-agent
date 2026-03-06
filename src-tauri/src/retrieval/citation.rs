use crate::retrieval::models::{Citation, RetrievedChunk};

pub fn citations(chunks: &[RetrievedChunk]) -> Vec<Citation> {
    chunks
        .iter()
        .map(|c| Citation {
            file: c.file.clone(),
            page_start: c.page_start,
            page_end: c.page_end,
            chunk_id: c.chunk_id.clone(),
            excerpt: c.text.chars().take(220).collect(),
        })
        .collect()
}
