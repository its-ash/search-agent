use std::sync::Arc;

use crate::{
    config::AppConfig,
    ingestion::coordinator::IngestionCoordinator,
    retrieval::query_engine::QueryEngine,
    server::lifecycle::ServerManager,
    storage::{keyword_store::KeywordStore, sqlite::SqliteStore, vector_store::VectorStore},
};

pub struct AppState {
    pub config: AppConfig,
    pub sqlite: Arc<SqliteStore>,
    pub vector_store: Arc<VectorStore>,
    pub keyword_store: Arc<KeywordStore>,
    pub server_manager: Arc<ServerManager>,
    pub ingestion: Arc<IngestionCoordinator>,
    pub query_engine: Arc<QueryEngine>,
}

impl AppState {
    pub async fn bootstrap(app: &tauri::AppHandle) -> anyhow::Result<Self> {
        let config = AppConfig::load(app)?;
        let sqlite = Arc::new(SqliteStore::connect(&config.sqlite_path).await?);
        sqlite.migrate().await?;

        let vector_store = Arc::new(VectorStore::new(&config.vector_dir)?);
        let keyword_store = Arc::new(KeywordStore::new(&config.keyword_dir)?);
        let server_manager = Arc::new(ServerManager::new(config.clone(), sqlite.clone()));

        let ingestion = Arc::new(IngestionCoordinator::new(
            sqlite.clone(),
            vector_store.clone(),
            keyword_store.clone(),
            server_manager.clone(),
        ));

        let query_engine = Arc::new(QueryEngine::new(
            sqlite.clone(),
            vector_store.clone(),
            keyword_store.clone(),
            server_manager.clone(),
        ));

        Ok(Self {
            config,
            sqlite,
            vector_store,
            keyword_store,
            server_manager,
            ingestion,
            query_engine,
        })
    }
}
