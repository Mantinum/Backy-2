// Repository module: store blobs and maintain an index

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{env, fs, io, path::PathBuf};
use uuid::Uuid;

/// A single entry in the repository index.
#[derive(Serialize, Deserialize)]
struct IndexEntry {
    id: Uuid,
    filename: String,
    length: usize,
}

/// Determine the repo directory and index JSON path.
fn get_paths() -> io::Result<(PathBuf, PathBuf)> {
    let base = env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|_| {
            ProjectDirs::from("com", "backy", "Backy")
                .map(|d| d.data_dir().to_path_buf())
                .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Cannot determine project directory"))
        })?;
    let repo_dir = base.join("repo");
    let index_file = repo_dir.join("index.json");
    Ok((repo_dir, index_file))
}

/// Initialize the repository directory and index file.
/// Creates directory and an empty index if missing.
pub fn init_repo() -> io::Result<(PathBuf, PathBuf)> {
    let (repo_dir, index_file) = get_paths()?;
    fs::create_dir_all(&repo_dir)?;
    if !index_file.exists() {
        fs::write(&index_file, "[]")?;
    }
    Ok((repo_dir, index_file))
}

/// Save a blob to the repository and append to the index.
/// Returns the assigned UUID.
pub fn save_blob(blob: &[u8]) -> io::Result<Uuid> {
    let (repo_dir, index_file) = init_repo()?;
    let id = Uuid::new_v4();
    let filename = format!("{}.blob", id);
    let blob_path = repo_dir.join(&filename);
    fs::write(&blob_path, blob)?;
    // Read and update index
    let mut entries: Vec<IndexEntry> = serde_json::from_slice(&fs::read(&index_file)?)?;
    entries.push(IndexEntry { id, filename, length: blob.len() });
    let new_index = serde_json::to_string_pretty(&entries)?;
    fs::write(&index_file, new_index)?;
    Ok(id)
}

/// List all blob UUIDs via the repository index.
pub fn list_blobs() -> io::Result<Vec<Uuid>> {
    let (_repo_dir, index_file) = init_repo()?;
    let entries: Vec<IndexEntry> = serde_json::from_slice(&fs::read(&index_file)?)?;
    Ok(entries.into_iter().map(|e| e.id).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_init_repo_empty() -> io::Result<()> {
        let temp = tempdir()?;
        unsafe { std::env::set_var("XDG_DATA_HOME", temp.path()); }
        let (repo_dir, index_file) = init_repo()?;
        assert!(repo_dir.exists());
        assert!(index_file.exists());
        let entries: Vec<IndexEntry> = serde_json::from_slice(&fs::read(&index_file)?)?;
        assert!(entries.is_empty());
        Ok(())
    }

    #[test]
    fn test_save_and_list_blobs() -> io::Result<()> {
        let temp = tempdir()?;
        unsafe { std::env::set_var("XDG_DATA_HOME", temp.path()); }
        let blob = b"hello".to_vec();
        let id = save_blob(&blob)?;
        let ids = list_blobs()?;
        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], id);
        Ok(())
    }
}
