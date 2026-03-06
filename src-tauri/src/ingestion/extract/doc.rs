use std::path::Path;

use tokio::process::Command;

pub async fn extract_doc_text(path: &Path) -> anyhow::Result<String> {
    let antiword = Command::new("antiword").arg(path).output().await;
    if let Ok(output) = antiword {
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
    }

    let soffice = Command::new("soffice")
        .args([
            "--headless",
            "--convert-to",
            "txt:Text",
            path.to_string_lossy().as_ref(),
            "--outdir",
            path.parent().and_then(|p| p.to_str()).unwrap_or("."),
        ])
        .output()
        .await;

    if let Ok(output) = soffice {
        if output.status.success() {
            let txt_path = path.with_extension("txt");
            if txt_path.exists() {
                let content = tokio::fs::read_to_string(&txt_path).await?;
                let _ = tokio::fs::remove_file(&txt_path).await;
                return Ok(content);
            }
        }
    }

    Err(anyhow::anyhow!(
        "DOC extraction failed: antiword/soffice unavailable or conversion error"
    ))
}
