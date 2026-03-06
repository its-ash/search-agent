use std::path::Path;

use tokio::process::Command;

pub async fn extract_via_ocr(path: &Path) -> anyhow::Result<String> {
    let out = Command::new("tesseract")
        .arg(path)
        .arg("stdout")
        .output()
        .await?;

    if !out.status.success() {
        return Err(anyhow::anyhow!("OCR failed; ensure tesseract is installed"));
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}
