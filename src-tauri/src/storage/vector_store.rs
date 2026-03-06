use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use parking_lot::Mutex;

use super::models::ChunkPayload;

pub struct VectorStore {
    file_path: PathBuf,
    lock: Mutex<()>,
}

impl VectorStore {
    pub fn new(base_dir: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(base_dir)?;
        let file_path = base_dir.join("vectors.jsonl");
        if !file_path.exists() {
            File::create(&file_path)?;
        }
        Ok(Self {
            file_path,
            lock: Mutex::new(()),
        })
    }

    pub async fn upsert_chunks(&self, chunks: &[ChunkPayload]) -> anyhow::Result<()> {
        let _g = self.lock.lock();
        let mut all = self.read_all()?;

        for chunk in chunks {
            if let Some(existing) = all.iter_mut().find(|c| c.chunk_id == chunk.chunk_id) {
                *existing = chunk.clone();
            } else {
                all.push(chunk.clone());
            }
        }

        self.write_all(&all)?;
        Ok(())
    }

    pub async fn delete_by_document(&self, document_id: &str) -> anyhow::Result<()> {
        let _g = self.lock.lock();
        let all = self.read_all()?;
        let filtered: Vec<_> = all
            .into_iter()
            .filter(|c| c.document_id != document_id)
            .collect();
        self.write_all(&filtered)?;
        Ok(())
    }

    pub async fn search(&self, query_embedding: &[f32], top_k: usize) -> anyhow::Result<Vec<(ChunkPayload, f32)>> {
        let _g = self.lock.lock();
        let all = self.read_all()?;

        let mut scored: Vec<(ChunkPayload, f32)> = all
            .into_iter()
            .map(|chunk| {
                let score = cosine_similarity(query_embedding, &chunk.embedding);
                (chunk, score)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored)
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        let _g = self.lock.lock();
        self.write_all(&[])?;
        Ok(())
    }

    fn read_all(&self) -> anyhow::Result<Vec<ChunkPayload>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut out = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            out.push(serde_json::from_str(&line)?);
        }
        Ok(out)
    }

    fn write_all(&self, chunks: &[ChunkPayload]) -> anyhow::Result<()> {
        let mut file = File::create(&self.file_path)?;
        for c in chunks {
            let line = serde_json::to_string(c)?;
            writeln!(file, "{line}")?;
        }
        Ok(())
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || b.is_empty() || a.len() != b.len() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}

#[cfg(test)]
mod tests {
    use super::cosine_similarity;

    #[test]
    fn cosine_similarity_basic() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!(cosine_similarity(&a, &b) > 0.99);
    }
}
