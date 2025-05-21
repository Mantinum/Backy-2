// Local storage destination: write blobs to a directory on the filesystem.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Save a blob to the given directory with the specified filename.
/// Creates the directory if it doesn't exist.
/// Returns the full path of the written file.
pub fn save_blob_local(
    blob: &[u8],
    dest_dir: &str,
    filename: &str,
) -> io::Result<String> {
    let dir = Path::new(dest_dir);
    fs::create_dir_all(dir)?;
    let mut path: PathBuf = dir.join(filename);
    // If no extension, default to .blob
    if path.extension().is_none() {
        path.set_extension("blob");
    }
    fs::write(&path, blob)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_save_blob_local() -> io::Result<()> {
        let dir = tempdir()?;
        let filename = "data123";
        let blob = b"local backup".to_vec();
        let saved = save_blob_local(&blob, dir.path().to_str().unwrap(), filename)?;
        assert!(saved.ends_with("data123.blob"));
        let loaded = fs::read(saved)?;
        assert_eq!(loaded, blob);
        Ok(())
    }
}
