use std::{collections::HashMap, path::Path};

use sha2::{Digest, Sha256};
use tokio::fs;

#[derive(Debug)]
pub struct DiffResult {
    pub changed_or_new: Vec<String>,
    pub deleted: Vec<String>,
}

pub async fn hash_file(path: &Path) -> anyhow::Result<String> {
    let data = fs::read(path).await?;
    let mut hasher = Sha256::new();
    hasher.update(data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn diff_paths(
    disk: &HashMap<String, (String, i64)>,
    db: &HashMap<String, (String, i64)>,
) -> DiffResult {
    let mut changed_or_new = Vec::new();
    let mut deleted = Vec::new();

    for (path, (hash, mtime)) in disk {
        match db.get(path) {
            None => changed_or_new.push(path.clone()),
            Some((old_hash, old_mtime)) => {
                if old_hash != hash || old_mtime != mtime {
                    changed_or_new.push(path.clone());
                }
            }
        }
    }

    for path in db.keys() {
        if !disk.contains_key(path) {
            deleted.push(path.clone());
        }
    }

    DiffResult {
        changed_or_new,
        deleted,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::diff_paths;

    #[test]
    fn detects_changed_and_deleted() {
        let mut disk = HashMap::new();
        disk.insert("a".into(), ("h1".into(), 1));

        let mut db = HashMap::new();
        db.insert("a".into(), ("old".into(), 1));
        db.insert("b".into(), ("h2".into(), 2));

        let diff = diff_paths(&disk, &db);
        assert_eq!(diff.changed_or_new, vec!["a"]);
        assert_eq!(diff.deleted, vec!["b"]);
    }
}
