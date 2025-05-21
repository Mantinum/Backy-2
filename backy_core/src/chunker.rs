// chunker module using FastCDC for content-defined chunking

use fastcdc::ronomon::FastCDC;
use std::fs::File;
use std::io::{self, Read};

/// Chunk a file at the given path using Content-Defined Chunking (FastCDC).
/// Returns a vector of byte vectors, each representing a chunk.
pub fn chunk_file(path: &str) -> io::Result<Vec<Vec<u8>>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    // chunk size parameters: min 2 MiB, avg 4 MiB, max 8 MiB
    let min_size = 1 << 21;
    let avg_size = 1 << 22;
    let max_size = 1 << 23;
    let mut chunks = Vec::new();
    for sec in FastCDC::new(&buffer, min_size, avg_size, max_size) {
        let start = sec.offset as usize;
        let end = start + sec.length as usize;
        chunks.push(buffer[start..end].to_vec());
    }
    Ok(chunks)
}
