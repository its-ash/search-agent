use crate::retrieval::models::RetrievedChunk;

pub fn rerank_by_overlap(question: &str, mut chunks: Vec<RetrievedChunk>) -> Vec<RetrievedChunk> {
    let q_terms: Vec<String> = question
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    for c in &mut chunks {
        let text = c.text.to_lowercase();
        let overlap = q_terms.iter().filter(|t| text.contains(t.as_str())).count() as f32;
        c.score += overlap * 0.01;
    }

    chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    chunks
}
