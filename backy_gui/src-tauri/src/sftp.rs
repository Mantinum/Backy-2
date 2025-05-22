use ssh2::Session;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use std::time::Duration;
use std::fs::File;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum SftpError {
    Io(io::Error),
    Ssh(ssh2::Error),
    Authentication(String),
    Connection(String),
    Operation(String),
}

impl fmt::Display for SftpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SftpError::Io(err) => write!(f, "IO error: {}", err),
            SftpError::Ssh(err) => write!(f, "SSH error: {}", err),
            SftpError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            SftpError::Connection(msg) => write!(f, "Connection error: {}", msg),
            SftpError::Operation(msg) => write!(f, "SFTP operation error: {}", msg),
        }
    }
}

impl Error for SftpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SftpError::Io(err) => Some(err),
            SftpError::Ssh(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for SftpError {
    fn from(err: io::Error) -> Self {
        SftpError::Io(err)
    }
}

impl From<ssh2::Error> for SftpError {
    fn from(err: ssh2::Error) -> Self {
        SftpError::Ssh(err)
    }
}

pub struct SftpClient {
    session: Session,
}

impl SftpClient {
    pub fn new(host: &str, port: u16, username: &str, password: &str) -> Result<Self, SftpError> {
        let tcp = TcpStream::connect((host, port)).map_err(|e| {
            SftpError::Connection(format!("Failed to connect to {}:{}: {}", host, port, e))
        })?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake().map_err(|e| SftpError::Ssh(e))?; // Or a more specific Connection error
        session.userauth_password(username, password).map_err(|e| SftpError::Ssh(e))?;

        if session.authenticated() {
            Ok(Self { session })
        } else {
            Err(SftpError::Authentication("Authentication failed".to_string()))
        }
    }

    pub fn upload_file(&self, local_path: &Path, remote_path: &str) -> Result<(), SftpError> {
        let sftp = self.session.sftp().map_err(|e| SftpError::Ssh(e))?;
        let mut local_file = std::fs::File::open(local_path)?;
        
        let remote_path_p = Path::new(remote_path);
        let mut remote_file = sftp.create(remote_path_p).map_err(|e| {
            SftpError::Operation(format!(
                "Failed to create remote file '{}': {}",
                remote_path_p.display(),
                e
            ))
        })?;

        let mut buffer = [0; 65536]; // 64KB buffer

        loop {
            let bytes_read = local_file.read(&mut buffer)?;
            if bytes_read == 0 {
                // End of file
                break;
            }
            remote_file.write_all(&buffer[..bytes_read]).map_err(|e| {
                SftpError::Operation(format!(
                    "Failed to write to remote file '{}': {}",
                    remote_path_p.display(),
                    e
                ))
            })?;
        }
        Ok(())
    }

    pub fn create_directory(&self, path: &str) -> Result<(), SftpError> {
        let sftp = self.session.sftp().map_err(|e| SftpError::Ssh(e))?;
        let path_p = Path::new(path);
        sftp.mkdir(path_p, 0o755).map_err(|e| {
            SftpError::Operation(format!(
                "Failed to create remote directory '{}': {}",
                path_p.display(),
                e
            ))
        })?;
        Ok(())
    }

    pub fn list_directory(&self, remote_path: &str) -> Result<Vec<String>, SftpError> {
        let sftp = self.session.sftp().map_err(|e| SftpError::Ssh(e))?;
        let path = Path::new(remote_path);
        let entries = sftp.readdir(path).map_err(|e| {
            SftpError::Operation(format!(
                "Failed to read directory '{}': {}",
                remote_path, e
            ))
        })?;

        let filenames = entries
            .into_iter()
            .map(|(entry_path, _stat)| {
                entry_path
                    .file_name()
                    .unwrap_or_default() // Use OsStr::new("") if file_name is None
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();

        Ok(filenames)
    }

    pub fn download_file(&self, remote_path: &str, local_path: &Path) -> Result<(), SftpError> {
        let sftp = self.session.sftp().map_err(|e| SftpError::Ssh(e))?;
        
        let mut remote_file = sftp.open(Path::new(remote_path)).map_err(|e| {
            SftpError::Operation(format!(
                "Failed to open remote file '{}': {}",
                remote_path, e
            ))
        })?;
        
        let mut local_file = File::create(local_path).map_err(|e| {
            SftpError::Io(e) // Or Operation if preferred for local file system interaction
        })?;

        let mut buffer = [0; 65536]; // 64KB buffer

        loop {
            let n = remote_file.read(&mut buffer).map_err(|e| {
                SftpError::Operation(format!(
                    "Failed to read from remote file '{}': {}",
                    remote_path, e
                ))
            })?;
            if n == 0 {
                // End of file
                break;
            }
            local_file.write_all(&buffer[..n]).map_err(|e| {
                SftpError::Io(e) // Or Operation for local write error
            })?;
        }
        Ok(())
    }
}
