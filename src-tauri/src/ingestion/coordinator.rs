use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use parking_lot::RwLock;
use tokio::{fs, sync::Semaphore};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    ingestion::{
        chunking, embeddings, enumerator,
        extract::extract_text,
        incremental::{diff_paths, hash_file},
        upsert,
    },
    server::lifecycle::ServerManager,
    storage::{
        keyword_store::KeywordStore,
        models::{ChunkPayload, DocumentRecord},
        sqlite::SqliteStore,
        vector_store::VectorStore,
    },
};

pub struct IngestionCoordinator {
    sqlite: Arc<SqliteStore>,
    vector: Arc<VectorStore>,
    keyword: Arc<KeywordStore>,
    server: Arc<ServerManager>,
    latest_job_id: RwLock<Option<String>>,
}

impl IngestionCoordinator {
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
            latest_job_id: RwLock::new(None),
        }
    }

    pub async fn start_scan(&self, root_path: String) -> anyhow::Result<String> {
        let root = PathBuf::from(root_path.clone());
        if !root.exists() || !root.is_dir() {
            return Err(anyhow::anyhow!("selected path is not a folder"));
        }

        let job_id = self.sqlite.create_scan_job(&root_path).await?;
        *self.latest_job_id.write() = Some(job_id.clone());

        let sqlite = self.sqlite.clone();
        let vector = self.vector.clone();
        let keyword = self.keyword.clone();
        let server = self.server.clone();
        let job_id_for_task = job_id.clone();

        tokio::spawn(async move {
            if let Err(err) = run_scan(sqlite, vector, keyword, server, &job_id_for_task, root).await {
                error!("scan job failed: {err}");
            }
        });

        Ok(job_id)
    }

    pub async fn cancel_scan(&self, job_id: &str) -> anyhow::Result<()> {
        self.sqlite.set_scan_canceled(job_id).await
    }

    pub async fn rebuild(&self, root_path: String) -> anyhow::Result<()> {
        self.start_scan(root_path).await?;
        Ok(())
    }

    pub async fn rebuild_last(&self) -> anyhow::Result<()> {
        let Some(root_path) = self.sqlite.latest_scan_root_path().await? else {
            return Err(anyhow::anyhow!("no previous scan available"));
        };
        self.start_scan(root_path).await?;
        Ok(())
    }

    pub async fn latest_job_id(&self) -> Option<String> {
        self.latest_job_id.read().clone()
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        self.vector.reset().await?;
        self.keyword.reset().await?;
        self.sqlite.reset_all().await?;
        Ok(())
    }
}

async fn run_scan(
    sqlite: Arc<SqliteStore>,
    vector: Arc<VectorStore>,
    keyword: Arc<KeywordStore>,
    server: Arc<ServerManager>,
    job_id: &str,
    root: PathBuf,
) -> anyhow::Result<()> {
    let files = enumerator::enumerate_supported_files(&root);
    sqlite.mark_scan_counts(job_id, files.len() as i64).await?;

    let mut disk_map = HashMap::new();
    for path in &files {
        let meta = fs::metadata(path).await?;
        let hash = hash_file(path).await?;
        let mtime = meta
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        disk_map.insert(path.to_string_lossy().to_string(), (hash, mtime));
    }

    let mut db_map = HashMap::new();
    for (_id, abs_path, file_hash, mtime_ms) in sqlite
        .get_documents_by_root(root.to_string_lossy().as_ref())
        .await?
    {
        db_map.insert(abs_path, (file_hash, mtime_ms));
    }

    let diff = diff_paths(&disk_map, &db_map);

    for deleted in diff.deleted {
        if let Some(doc_id) = sqlite.delete_document_by_path(&deleted).await? {
            vector.delete_by_document(&doc_id).await?;
        }
    }

    let sem = Arc::new(Semaphore::new(4));
    let mut processed = 0i64;
    let mut failed = 0i64;

    for path_str in diff.changed_or_new {
        if sqlite.is_scan_canceled(job_id).await? {
            sqlite.complete_scan(job_id, "canceled", None).await?;
            return Ok(());
        }

        sqlite
            .update_scan_progress(job_id, processed, failed, Some(&path_str), "indexing")
            .await?;

        let _permit = sem.clone().acquire_owned().await?;
        let path = PathBuf::from(&path_str);

        match index_one_file(&sqlite, &vector, &keyword, &server, &root, &path).await {
            Ok(_) => processed += 1,
            Err(err) => {
                failed += 1;
                sqlite
                    .add_failure(job_id, &path_str, "index", &err.to_string(), true)
                    .await?;
            }
        }

        sqlite
            .update_scan_progress(job_id, processed, failed, None, "indexing")
            .await?;
    }

    let final_state = if failed > 0 { "ready" } else { "ready" };
    sqlite.complete_scan(job_id, final_state, None).await?;
    info!("scan completed job={job_id}");
    Ok(())
}

async fn index_one_file(
    sqlite: &SqliteStore,
    vector: &VectorStore,
    keyword: &KeywordStore,
    server: &ServerManager,
    root: &Path,
    path: &Path,
) -> anyhow::Result<()> {
    let metadata = fs::metadata(path).await?;
    let mtime_ms = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let file_hash = hash_file(path).await?;
    let abs_path = path.canonicalize()?.to_string_lossy().to_string();
    let rel_path = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    let text = extract_text(path).await?;
    if text.trim().is_empty() {
        return Err(anyhow::anyhow!("no text extracted"));
    }

    let chunks = chunking::chunk_text(&text, 220, 40);
    let chunk_texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();

    let embed_model = embeddings::default_embedding_model();
    let embeddings = embeddings::embed_batch(server, &chunk_texts, &embed_model).await?;

    if embeddings.len() != chunk_texts.len() {
        return Err(anyhow::anyhow!("embedding count mismatch"));
    }

    let doc_id = Uuid::new_v4().to_string();
    let doc = DocumentRecord {
        id: doc_id.clone(),
        abs_path: abs_path.clone(),
        rel_path: rel_path.clone(),
        file_ext: path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase(),
        file_hash,
        mtime_ms,
        size_bytes: metadata.len() as i64,
    };

    sqlite.upsert_document(&doc).await?;

    let mut payloads = Vec::new();
    let mut keyword_docs = Vec::new();

    for (idx, (chunk, embedding)) in chunks.into_iter().zip(embeddings.into_iter()).enumerate() {
        let chunk_id = format!("{doc_id}:{idx}");
        keyword_docs.push((chunk_id.clone(), chunk.text.clone()));
        payloads.push(ChunkPayload {
            chunk_id,
            document_id: doc_id.clone(),
            abs_path: abs_path.clone(),
            rel_path: rel_path.clone(),
            page_start: chunk.page_start,
            page_end: chunk.page_end,
            section: chunk.section,
            text: chunk.text,
            embedding,
        });
    }

    upsert::upsert_chunks(sqlite, vector, &doc_id, payloads).await?;
    keyword.upsert(&keyword_docs).await?;

    Ok(())
}
