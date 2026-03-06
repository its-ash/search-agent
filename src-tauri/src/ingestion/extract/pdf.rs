use std::path::Path;

use super::ocr;

pub async fn extract_pdf_text(path: &Path) -> anyhow::Result<String> {
    let path_buf = path.to_path_buf();
    let extracted = tokio::task::spawn_blocking(move || pdf_extract::extract_text(&path_buf)).await??;

    if extracted.trim().is_empty() {
        return ocr::extract_via_ocr(path).await;
    }

    Ok(extracted)
}
