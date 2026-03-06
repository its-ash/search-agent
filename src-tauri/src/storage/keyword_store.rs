use std::path::Path;

use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Value, STORED, STRING, Schema, TEXT},
    Index,
};

pub struct KeywordStore {
    index: Index,
    id_field: Field,
    text_field: Field,
}

impl KeywordStore {
    pub fn new(base_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(base_dir)?;

        let mut schema_builder = Schema::builder();
        let id_field = schema_builder.add_text_field("chunk_id", STRING | STORED);
        let text_field = schema_builder.add_text_field("text", TEXT | STORED);
        let schema = schema_builder.build();

        let index = match Index::open_in_dir(base_dir) {
            Ok(ix) => ix,
            Err(_) => Index::create_in_dir(base_dir, schema)?,
        };

        Ok(Self {
            index,
            id_field,
            text_field,
        })
    }

    pub async fn upsert(&self, chunks: &[(String, String)]) -> anyhow::Result<()> {
        let mut writer = self.index.writer::<tantivy::TantivyDocument>(20_000_000)?;
        for (id, text) in chunks {
            writer.add_document(tantivy::doc!(
                self.id_field => id.clone(),
                self.text_field => text.clone()
            ))?;
        }
        writer.commit()?;
        Ok(())
    }

    pub async fn query(&self, q: &str, k: usize) -> anyhow::Result<Vec<String>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let parser = QueryParser::for_index(&self.index, vec![self.text_field]);
        let query = parser.parse_query(q)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(k))?;

        let mut out = Vec::new();
        for (_score, addr) in top_docs {
            let doc = searcher.doc::<tantivy::TantivyDocument>(addr)?;
            if let Some(value) = doc.get_first(self.id_field) {
                if let Some(v) = value.as_str() {
                    out.push(v.to_string());
                }
            }
        }

        Ok(out)
    }

    pub async fn reset(&self) -> anyhow::Result<()> {
        let mut writer = self.index.writer::<tantivy::TantivyDocument>(20_000_000)?;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }
}
