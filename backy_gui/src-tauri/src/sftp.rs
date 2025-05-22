use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::io::{self, Read, Write};
use std::time::Duration;

pub struct SftpClient {
    session: Session,
}

impl SftpClient {
    pub fn new(host: &str, port: u16, username: &str, password: &str) -> io::Result<Self> {
        let tcp = TcpStream::connect((host, port))?;
        tcp.set_read_timeout(Some(Duration::from_secs(10)))?;
        tcp.set_write_timeout(Some(Duration::from_secs(10)))?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_password(username, password)?;

        if session.authenticated() {
            Ok(Self { session })
        } else {
            Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Authentication failed",
            ))
        }
    }

    pub fn upload_file(&self, local_path: &Path, remote_path: &str) -> io::Result<()> {
        let sftp = self.session.sftp()?;
        let mut file = std::fs::File::open(local_path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;

        let remote_path = Path::new(remote_path);
        let mut remote_file = sftp.create(remote_path)?;
        remote_file.write_all(&contents)?;
        Ok(())
    }

    pub fn create_directory(&self, path: &str) -> io::Result<()> {
        let sftp = self.session.sftp()?;
        let path = Path::new(path);
        sftp.mkdir(path, 0o755)?;
        Ok(())
    }
}
