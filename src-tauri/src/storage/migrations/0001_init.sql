CREATE TABLE IF NOT EXISTS schema_version (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS server_state (
  id INTEGER PRIMARY KEY CHECK(id = 1),
  ownership TEXT NOT NULL,
  pid INTEGER,
  endpoint TEXT NOT NULL,
  last_health_at TEXT,
  status TEXT NOT NULL,
  message TEXT
);

CREATE TABLE IF NOT EXISTS scan_jobs (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  total_files INTEGER DEFAULT 0,
  processed_files INTEGER DEFAULT 0,
  failed_files INTEGER DEFAULT 0,
  canceled INTEGER DEFAULT 0,
  current_file TEXT,
  error TEXT
);

CREATE TABLE IF NOT EXISTS documents (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  rel_path TEXT NOT NULL,
  abs_path TEXT NOT NULL UNIQUE,
  file_ext TEXT NOT NULL,
  file_hash TEXT NOT NULL,
  mtime_ms INTEGER NOT NULL,
  size_bytes INTEGER NOT NULL,
  page_count INTEGER,
  extractor TEXT,
  status TEXT NOT NULL,
  last_indexed_at TEXT,
  last_error TEXT
);

CREATE TABLE IF NOT EXISTS document_chunks (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  chunk_index INTEGER NOT NULL,
  page_start INTEGER,
  page_end INTEGER,
  section TEXT,
  token_count INTEGER NOT NULL,
  text_preview TEXT,
  vector_ref TEXT NOT NULL,
  keyword_ref TEXT,
  FOREIGN KEY(document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ingestion_failures (
  id TEXT PRIMARY KEY,
  scan_job_id TEXT NOT NULL,
  abs_path TEXT NOT NULL,
  stage TEXT NOT NULL,
  error_code TEXT NOT NULL,
  error_message TEXT NOT NULL,
  retryable INTEGER NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS query_logs (
  id TEXT PRIMARY KEY,
  question TEXT NOT NULL,
  answer TEXT,
  latency_ms INTEGER,
  context_chunk_ids TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_abs_path ON documents(abs_path);
CREATE INDEX IF NOT EXISTS idx_documents_hash_mtime ON documents(file_hash, mtime_ms);
CREATE INDEX IF NOT EXISTS idx_chunks_document ON document_chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_failures_job ON ingestion_failures(scan_job_id);
