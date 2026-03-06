use std::path::PathBuf;

use anyhow::Context;
use tauri::AppHandle;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub data_dir: PathBuf,
    pub sqlite_path: PathBuf,
    pub vector_dir: PathBuf,
    pub keyword_dir: PathBuf,
    pub log_dir: PathBuf,
    pub llama_endpoint: String,
    pub llama_command: String,
    pub llama_args: Vec<String>,
    pub embedding_endpoint: String,
}

impl AppConfig {
    pub fn load(app: &AppHandle) -> anyhow::Result<Self> {
        let _ = app;
        let home = std::env::var("HOME").context("HOME is not set")?;
        let data_dir = PathBuf::from(home).join(".search-agent");
        let sqlite_path = data_dir.join("metadata.sqlite3");
        let vector_dir = data_dir.join("vector_index");
        let keyword_dir = data_dir.join("keyword_index");
        let log_dir = data_dir.join("logs");

        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&vector_dir)?;
        std::fs::create_dir_all(&keyword_dir)?;
        std::fs::create_dir_all(&log_dir)?;

        Ok(Self {
            data_dir,
            sqlite_path,
            vector_dir,
            keyword_dir,
            log_dir,
            llama_endpoint: "http://127.0.0.1:11434".to_string(),
            llama_command: "ollama".to_string(),
            llama_args: vec![
                "serve".to_string(),
            ],
            embedding_endpoint: "http://127.0.0.1:11434/api/embeddings".to_string(),
        })
    }
}
