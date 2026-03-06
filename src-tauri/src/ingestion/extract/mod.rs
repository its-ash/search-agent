pub mod doc;
pub mod docx;
pub mod ocr;
pub mod pdf;

use std::path::Path;

pub async fn extract_text(path: &Path) -> anyhow::Result<String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();

    match ext.as_str() {
        "pdf" => pdf::extract_pdf_text(path).await,
        "docx" => docx::extract_docx_text(path).await,
        "doc" => doc::extract_doc_text(path).await,
        _ => extract_plain_text(path).await,
    }
}

async fn extract_plain_text(path: &Path) -> anyhow::Result<String> {
    let bytes = tokio::fs::read(path).await?;
    if bytes.is_empty() {
        return Ok(String::new());
    }

    match String::from_utf8(bytes) {
        Ok(text) => Ok(text),
        Err(err) => {
            let lossy = String::from_utf8_lossy(err.as_bytes()).to_string();
            let printable = lossy
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
                .count();
            let ratio = printable as f32 / lossy.chars().count().max(1) as f32;
            if ratio > 0.85 {
                Ok(lossy)
            } else {
                Err(anyhow::anyhow!("likely binary file; skipped"))
            }
        }
    }
}
