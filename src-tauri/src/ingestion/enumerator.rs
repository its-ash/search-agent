use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn is_supported_extension(ext: &str) -> bool {
    const TEXT_EXTS: &[&str] = &[
        "txt", "md", "rst", "csv", "json", "yaml", "yml", "toml", "xml", "html", "htm",
        "js", "jsx", "ts", "tsx", "py", "java", "rs", "go", "c", "cpp", "h", "hpp", "cs",
        "php", "rb", "swift", "kt", "kts", "scala", "sql", "sh", "bash", "zsh", "ps1",
        "env", "ini", "cfg", "conf", "log",
    ];
    ext == "pdf" || ext == "docx" || ext == "doc" || TEXT_EXTS.contains(&ext)
}

pub fn enumerate_supported_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(ext) = path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) else {
            continue;
        };
        if is_supported_extension(&ext) {
            out.push(path.to_path_buf());
        }
    }
    out
}
