use std::path::Path;

use anyhow::Context;
use chrono::Utc;
use sqlx::{
    migrate::Migrator,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    ConnectOptions, Pool, Sqlite,
};
use uuid::Uuid;

use super::models::{DocumentRecord, FailureEntry, ScanStatus};

static MIGRATOR: Migrator = sqlx::migrate!("./src/storage/migrations");

pub struct SqliteStore {
    pool: Pool<Sqlite>,
}

impl SqliteStore {
    pub async fn connect(path: &Path) -> anyhow::Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .disable_statement_logging();

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await
            .context("failed to connect sqlite")?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        MIGRATOR.run(&self.pool).await?;
        Ok(())
    }

    pub async fn create_scan_job(&self, root_path: &str) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO scan_jobs (id, root_path, status, started_at, total_files, processed_files, failed_files, canceled) VALUES (?1, ?2, 'scanning', ?3, 0, 0, 0, 0)",
        )
        .bind(&id)
        .bind(root_path)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn mark_scan_counts(&self, job_id: &str, total: i64) -> anyhow::Result<()> {
        sqlx::query("UPDATE scan_jobs SET total_files=?1 WHERE id=?2")
            .bind(total)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_scan_progress(
        &self,
        job_id: &str,
        processed: i64,
        failed: i64,
        current_file: Option<&str>,
        status: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE scan_jobs SET processed_files=?1, failed_files=?2, current_file=?3, status=?4 WHERE id=?5",
        )
        .bind(processed)
        .bind(failed)
        .bind(current_file)
        .bind(status)
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn complete_scan(&self, job_id: &str, status: &str, err: Option<&str>) -> anyhow::Result<()> {
        sqlx::query("UPDATE scan_jobs SET status=?1, finished_at=?2, error=?3, current_file=NULL WHERE id=?4")
            .bind(status)
            .bind(Utc::now().to_rfc3339())
            .bind(err)
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_scan_canceled(&self, job_id: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE scan_jobs SET canceled=1, status='canceled' WHERE id=?1")
            .bind(job_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_scan_canceled(&self, job_id: &str) -> anyhow::Result<bool> {
        let row: (i64,) = sqlx::query_as("SELECT canceled FROM scan_jobs WHERE id=?1")
            .bind(job_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.0 == 1)
    }

    pub async fn get_scan_status(&self, job_id: &str) -> anyhow::Result<ScanStatus> {
        let row = sqlx::query_as::<_, (String, String, i64, i64, i64, Option<String>)>(
            "SELECT id, status, total_files, processed_files, failed_files, current_file FROM scan_jobs WHERE id=?1",
        )
        .bind(job_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(ScanStatus {
            job_id: row.0,
            status: row.1,
            total_files: row.2,
            processed_files: row.3,
            failed_files: row.4,
            current_file: row.5,
        })
    }

    pub async fn add_failure(
        &self,
        job_id: &str,
        path: &str,
        stage: &str,
        message: &str,
        retryable: bool,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO ingestion_failures (id, scan_job_id, abs_path, stage, error_code, error_message, retryable, created_at) VALUES (?1, ?2, ?3, ?4, 'E_INGEST', ?5, ?6, ?7)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(job_id)
        .bind(path)
        .bind(stage)
        .bind(message)
        .bind(if retryable { 1 } else { 0 })
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_failures(&self, job_id: &str) -> anyhow::Result<Vec<FailureEntry>> {
        let rows = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT abs_path, stage, error_message, retryable FROM ingestion_failures WHERE scan_job_id=?1 ORDER BY created_at DESC",
        )
        .bind(job_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| FailureEntry {
                path: r.0,
                stage: r.1,
                message: r.2,
                retryable: r.3 == 1,
            })
            .collect())
    }

    pub async fn upsert_document(&self, doc: &DocumentRecord) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO documents (id, root_path, rel_path, abs_path, file_ext, file_hash, mtime_ms, size_bytes, status, last_indexed_at)
             VALUES (?1, '', ?2, ?3, ?4, ?5, ?6, ?7, 'indexed', ?8)
             ON CONFLICT(abs_path) DO UPDATE SET file_hash=excluded.file_hash, mtime_ms=excluded.mtime_ms, size_bytes=excluded.size_bytes, status='indexed', last_indexed_at=excluded.last_indexed_at",
        )
        .bind(&doc.id)
        .bind(&doc.rel_path)
        .bind(&doc.abs_path)
        .bind(&doc.file_ext)
        .bind(&doc.file_hash)
        .bind(doc.mtime_ms)
        .bind(doc.size_bytes)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_documents_by_root(&self, root: &str) -> anyhow::Result<Vec<(String, String, String, i64)>> {
        let like = format!("{root}%");
        let rows = sqlx::query_as::<_, (String, String, String, i64)>(
            "SELECT id, abs_path, file_hash, mtime_ms FROM documents WHERE abs_path LIKE ?1",
        )
        .bind(like)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn delete_document_by_path(&self, abs_path: &str) -> anyhow::Result<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>("SELECT id FROM documents WHERE abs_path=?1")
            .bind(abs_path)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((doc_id,)) = row {
            sqlx::query("DELETE FROM documents WHERE id=?1")
                .bind(&doc_id)
                .execute(&self.pool)
                .await?;
            return Ok(Some(doc_id));
        }
        Ok(None)
    }

    pub async fn set_server_state(
        &self,
        status: &str,
        ownership: &str,
        pid: Option<i64>,
        endpoint: &str,
        message: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO server_state (id, ownership, pid, endpoint, last_health_at, status, message)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET ownership=excluded.ownership, pid=excluded.pid, endpoint=excluded.endpoint, last_health_at=excluded.last_health_at, status=excluded.status, message=excluded.message",
        )
        .bind(ownership)
        .bind(pid)
        .bind(endpoint)
        .bind(Utc::now().to_rfc3339())
        .bind(status)
        .bind(message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn reset_all(&self) -> anyhow::Result<()> {
        for table in [
            "document_chunks",
            "documents",
            "ingestion_failures",
            "query_logs",
            "scan_jobs",
        ] {
            let q = format!("DELETE FROM {table}");
            sqlx::query(&q).execute(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn save_chunk_refs(&self, document_id: &str, chunk_ids: &[String]) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM document_chunks WHERE document_id=?1")
            .bind(document_id)
            .execute(&self.pool)
            .await?;

        for (idx, cid) in chunk_ids.iter().enumerate() {
            sqlx::query("INSERT INTO document_chunks (id, document_id, chunk_index, token_count, text_preview, vector_ref) VALUES (?1, ?2, ?3, 0, '', ?4)")
                .bind(cid)
                .bind(document_id)
                .bind(idx as i64)
                .bind(cid)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn latest_scan_root_path(&self) -> anyhow::Result<Option<String>> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT root_path FROM scan_jobs ORDER BY started_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn get_index_counts(&self) -> anyhow::Result<(i64, i64)> {
        let (files,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM documents")
            .fetch_one(&self.pool)
            .await?;
        let (chunks,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM document_chunks")
            .fetch_one(&self.pool)
            .await?;
        Ok((files, chunks))
    }
}
