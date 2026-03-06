use std::{sync::Arc, time::Instant};

use chrono::Utc;
use serde_json::json;

use crate::{
    commands::query::{AskRequest, AskResponse},
    retrieval::{citation, extractive, hybrid, prompt_builder, query_embed, rerank, vector_search},
    server::lifecycle::ServerManager,
    storage::{keyword_store::KeywordStore, sqlite::SqliteStore, vector_store::VectorStore},
};

pub struct QueryEngine {
    sqlite: Arc<SqliteStore>,
    vector: Arc<VectorStore>,
    keyword: Arc<KeywordStore>,
    server: Arc<ServerManager>,
}

impl QueryEngine {
    pub fn new(
        sqlite: Arc<SqliteStore>,
        vector: Arc<VectorStore>,
        keyword: Arc<KeywordStore>,
        server: Arc<ServerManager>,
    ) -> Self {
        Self {
            sqlite,
            vector,
            keyword,
            server,
        }
    }

    pub async fn ask(&self, req: AskRequest) -> anyhow::Result<AskResponse> {
        let started = Instant::now();
        let top_k = req.top_k.unwrap_or(6).max(1);

        let q_embed = query_embed::embed_query(&self.server, &req.question).await?;
        let mut retrieved = vector_search::top_k(&self.vector, &q_embed, top_k).await?;

        if req.hybrid.unwrap_or(true) {
            retrieved = hybrid::merge_with_keyword(retrieved, &self.keyword, &req.question, top_k).await?;
        }

        if req.rerank.unwrap_or(true) {
            retrieved = rerank::rerank_by_overlap(&req.question, retrieved);
        }

        let filtered: Vec<_> = retrieved
            .into_iter()
            .filter(|c| c.score > 0.0 && !c.text.trim().is_empty())
            .collect();

        if filtered.is_empty() {
            return Ok(AskResponse {
                answer: "Not found in indexed documents.".to_string(),
                not_found: true,
                citations: vec![],
                latency_ms: started.elapsed().as_millis() as i64,
            });
        }

        if let Some(extractive_answer) = extractive::answer_from_context(&req.question, &filtered) {
            let source = &filtered[extractive_answer.source_chunk_index..=extractive_answer.source_chunk_index];
            let citations = citation::citations(source);
            return Ok(AskResponse {
                answer: extractive_answer.answer,
                not_found: false,
                citations,
                latency_ms: started.elapsed().as_millis() as i64,
            });
        }

        let prompt = prompt_builder::build_prompt(&req.question, &filtered);
        let mut answer = call_chat_completion(self.server.endpoint(), &prompt).await?;
        if answer.trim() == "Not found in indexed documents." {
            if let Some(extractive_answer) = extractive::answer_from_context(&req.question, &filtered) {
                answer = extractive_answer.answer;
            }
        }

        let not_found = answer.trim() == "Not found in indexed documents.";
        let citations = citation::citations(&filtered[..filtered.len().min(3)]);

        let _ = sqlx::query("INSERT INTO query_logs (id, question, answer, latency_ms, context_chunk_ids, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .bind(uuid::Uuid::new_v4().to_string())
            .bind(&req.question)
            .bind(&answer)
            .bind(started.elapsed().as_millis() as i64)
            .bind(serde_json::to_string(&citations.iter().map(|c| c.chunk_id.clone()).collect::<Vec<_>>())?)
            .bind(Utc::now().to_rfc3339())
            .execute(self.sqlite.pool())
            .await;

        Ok(AskResponse {
            answer,
            not_found,
            citations,
            latency_ms: started.elapsed().as_millis() as i64,
        })
    }
}

async fn call_chat_completion(endpoint: &str, prompt: &str) -> anyhow::Result<String> {
    let model = std::env::var("SEARCH_AGENT_CHAT_MODEL").unwrap_or_else(|_| "llama3:latest".to_string());
    let url = format!("{endpoint}/v1/chat/completions");
    let body = json!({
        "model": model,
        "temperature": 0.0,
        "max_tokens": 220,
        "messages": [
            {"role": "system", "content": "You are a retrieval QA assistant. Use only supplied context."},
            {"role": "user", "content": prompt}
        ]
    });

    let client = reqwest::Client::new();
    let resp = client.post(url).json(&body).send().await;

    match resp {
        Ok(r) if r.status().is_success() => {
            let value: serde_json::Value = r.json().await?;
            if let Some(content) = value["choices"][0]["message"]["content"].as_str() {
                Ok(content.to_string())
            } else {
                call_ollama_chat(endpoint, prompt, &model).await
            }
        }
        _ => call_ollama_chat(endpoint, prompt, &model).await,
    }
}

async fn call_ollama_chat(endpoint: &str, prompt: &str, model: &str) -> anyhow::Result<String> {
    let url = format!("{endpoint}/api/chat");
    let body = json!({
        "model": model,
        "stream": false,
        "options": {
            "temperature": 0.0,
            "num_predict": 220
        },
        "messages": [
            {"role": "system", "content": "You are a retrieval QA assistant. Use only supplied context."},
            {"role": "user", "content": prompt}
        ]
    });

    let client = reqwest::Client::new();
    let resp = client.post(url).json(&body).send().await;
    match resp {
        Ok(r) if r.status().is_success() => {
            let value: serde_json::Value = r.json().await?;
            if let Some(content) = value["message"]["content"].as_str() {
                Ok(content.to_string())
            } else {
                Ok("Not found in indexed documents.".to_string())
            }
        }
        _ => Ok("Not found in indexed documents.".to_string()),
    }
}
