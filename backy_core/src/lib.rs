mod chunker;
pub use chunker::chunk_file;

mod crypto;
pub use crypto::{encrypt, decrypt};

mod repository;
pub use repository::{init_repo, save_blob, list_blobs};

mod storage_local;
pub use storage_local::save_blob_local;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

pub fn backup_start(source: &str) -> Result<String, std::io::Error> {
    let output = std::process::Command::new("kopia")
        .args(&["snapshot", "create", source, "--json"])
        .output()?;
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("kopia failed with exit code: {}", output.status),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn backup_start_error() {
        let result = backup_start("nonexistent_path");
        assert!(result.is_err());
    }
}
