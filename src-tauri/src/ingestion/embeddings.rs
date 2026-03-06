use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::server::lifecycle::ServerManager;

#[derive(Debug, Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingDatum {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
struct OllamaEmbeddingsRequest<'a> {
    model: &'a str,
    input: Vec<&'a str>,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbeddingsResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Debug, Serialize)]
struct OllamaLegacyEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Debug, Deserialize)]
struct OllamaLegacyEmbeddingResponse {
    embedding: Vec<f32>,
}

pub fn default_embedding_model() -> String {
    std::env::var("SEARCH_AGENT_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string())
}

pub async fn embed_batch(
    server: &ServerManager,
    texts: &[String],
    model: &str,
) -> anyhow::Result<Vec<Vec<f32>>> {
    if texts.is_empty() {
        return Ok(vec![]);
    }

    let openai_endpoint = format!("{}/v1/embeddings", server.endpoint());
    let req = EmbeddingRequest {
        model,
        input: texts.iter().map(String::as_str).collect(),
    };

    let client = reqwest::Client::new();
    let result = client.post(openai_endpoint).json(&req).send().await;
    match result {
        Ok(resp) if resp.status().is_success() => {
            let parsed: EmbeddingResponse = resp.json().await?;
            finalize_embeddings(parsed.data.into_iter().map(|d| d.embedding).collect())
        }
        _ => {
            match embed_with_ollama_batch(&client, server.endpoint(), texts, model).await {
                Ok(vectors) if !vectors.is_empty() => return finalize_embeddings(vectors),
                _ => {}
            }
            match embed_with_ollama_legacy(&client, server.endpoint(), texts, model).await {
                Ok(vectors) if !vectors.is_empty() => return finalize_embeddings(vectors),
                _ => {}
            }

            if allow_fake_embeddings() {
                warn!("using deterministic fallback embeddings; retrieval quality will be degraded");
                let fallback = texts.iter().map(|t| deterministic_embedding(t, 384)).collect();
                return finalize_embeddings(fallback);
            }

            Err(anyhow::anyhow!(
                "embedding generation failed. Ensure Ollama is running and model '{}' is available (try: ollama pull {}).",
                model,
                model
            ))
        }
    }
}

async fn embed_with_ollama_batch(
    client: &reqwest::Client,
    endpoint: &str,
    texts: &[String],
    model: &str,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let url = format!("{endpoint}/api/embed");
    let body = OllamaEmbeddingsRequest {
        model,
        input: texts.iter().map(String::as_str).collect(),
    };

    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("ollama /api/embed request failed"));
    }
    let parsed: OllamaEmbeddingsResponse = resp.json().await?;
    Ok(parsed.embeddings)
}

async fn embed_with_ollama_legacy(
    client: &reqwest::Client,
    endpoint: &str,
    texts: &[String],
    model: &str,
) -> anyhow::Result<Vec<Vec<f32>>> {
    let mut out = Vec::with_capacity(texts.len());
    let url = format!("{endpoint}/api/embeddings");

    for text in texts {
        let body = OllamaLegacyEmbeddingRequest { model, prompt: text };
        let resp = client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("ollama /api/embeddings request failed"));
        }
        let parsed: OllamaLegacyEmbeddingResponse = resp.json().await?;
        out.push(parsed.embedding);
    }

    Ok(out)
}

fn finalize_embeddings(mut vectors: Vec<Vec<f32>>) -> anyhow::Result<Vec<Vec<f32>>> {
    let dim = vectors.first().map(Vec::len).unwrap_or(0);
    if dim == 0 {
        return Err(anyhow::anyhow!("empty embedding vectors"));
    }

    for v in &mut vectors {
        if v.len() != dim {
            return Err(anyhow::anyhow!("embedding dimensions mismatch"));
        }
        normalize(v);
    }

    Ok(vectors)
}

fn normalize(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in v.iter_mut() {
            *x /= norm;
        }
    }
}

fn allow_fake_embeddings() -> bool {
    std::env::var("SEARCH_AGENT_ALLOW_FAKE_EMBEDDINGS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn deterministic_embedding(text: &str, dims: usize) -> Vec<f32> {
    let mut out = vec![0f32; dims];
    for (i, b) in text.as_bytes().iter().enumerate() {
        out[i % dims] += (*b as f32) / 255.0;
    }
    out
}
