# SearchAgent (Tauri Local-First RAG)

Production-oriented local-first RAG desktop app built with Tauri.

## What it includes
- Folder picker + scan/index UI + progress + error panel + chat + citations panel.
- Tauri commands:
  - `start_scan`
  - `get_scan_status`
  - `cancel_scan`
  - `ask_question`
  - `get_server_status`
  - `restart_server`
  - `rebuild_index`
  - `reset_index`
- Rust backend pipelines:
  - Recursive ingestion for `pdf/docx/doc`
  - OCR fallback for image-only PDFs (`tesseract`)
  - Token-aware chunking with overlap
  - Incremental indexing with hash + mtime + delete detection
  - Local vector index (file-backed), optional keyword retrieval (Tantivy)
  - Grounded prompt building with prompt-injection guard
- Llama server lifecycle ownership:
  - Reuse existing server if already healthy
  - Spawn only when not running
  - Stop on app close only if this app started it
- Persistence:
  - SQLite metadata + scan jobs + server state + query logs
  - Migration files in `src-tauri/src/storage/migrations`
- Tests:
  - Chunking
  - Hash-change detection
  - Ownership shutdown logic
  - Integration-style scan/index/query path

## Prerequisites
- Node 20+
- Rust stable
- Tauri prerequisites for your OS
- Ollama installed and available in PATH
  - Verify: `ollama --version`
  - Start server (if not auto-running): `ollama serve`
  - Recommended models:
    - `ollama pull llama3:latest`
    - `ollama pull nomic-embed-text`
- Optional for better document support:
  - `tesseract` (OCR)
  - `antiword` and/or `soffice` for `.doc`

## Run
```bash
npm install
npm run tauri dev
```

Optional validation:
```bash
cd src-tauri && cargo test && cd ..
```

## Build desktop app
```bash
npm run build
cargo tauri build
```
