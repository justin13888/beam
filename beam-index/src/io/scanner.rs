use futures::stream::{FuturesUnordered, StreamExt};
use memmap2::MmapOptions;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;
use tokio::sync::Semaphore;
use tokio::task;
use url::Url;
use walkdir::WalkDir;

const MIN_MULTITHREADED_HASHING_SIZE: u64 = 128 * 1024; // 128 KiB

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub last_modified: u64,
    pub created: u64,
    pub hash: Vec<u8>,
    // TODO: Add more metadata
}

// #[derive(Debug, Clone)]
// pub enum MediaType {
//     Movie(MovieMetadata),
//     TVShow(TVShowMetadata),
//     Unknown,
//     // TODO: Implement more specific types
// }

// #[derive(Debug, Clone)]
// pub struct MovieMetadata {
//     pub title: String,
//     pub year: u16,
//     pub description: String,
//     pub genres: Vec<String>,
//     pub rating: f32,
//     /// Runtime in minutes
//     pub runtime: Duration,
//     // pub plot: String,
//     // pub poster: String,
//     pub trailer_url: Option<Url>,
//     // TODO: Whatever else is provided by TMDB
// } // TODO: Implement

// #[derive(Debug, Clone)]
// pub struct TVShowMetadata; // TODO: Implement

#[derive(Error, Debug)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("File metadata error: {0}")]
    Metadata(String),
}

pub type ScanResult = Vec<Result<FileMetadata, ScanError>>;

/// Scans a directory for media files
/// Returns a list of metadata for every file
pub async fn scan_media(media_path: &Path, max_simulataneous_scan: usize, max_partial_hash: usize) -> ScanResult {
    // Detect for file changes by name and hash
    // TODO: Compare difference and associate with

    let semaphore = Arc::new(Semaphore::new(max_simulataneous_scan)); // Limit to 10 concurrent scans
    let mut tasks = FuturesUnordered::new();

    let walker = WalkDir::new(media_path).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.into_path();
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let task = task::spawn(async move {
                let result = scan_file(path, max_partial_hash).await;
                drop(permit);
                result
            });
            tasks.push(task);
        }
    }

    let mut results: ScanResult = Vec::new();

    while let Some(result) = tasks.next().await {
        results.push(result.unwrap());
    }

    results
}

async fn scan_file(file_path: PathBuf, max_partial_hash: usize) -> Result<FileMetadata, ScanError> {
    let mut file = File::open(&file_path)?;
    let metadata = file.metadata()?;
    let size = metadata.len();

    // TODO: These two lines below look wrong
    let last_modified = metadata.modified().unwrap().duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .as_secs();
    let created = metadata.created().unwrap().duration_since(UNIX_EPOCH)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .as_secs();

    // Hash the file
    // let partial_mmap = unsafe { MmapOptions::new().len(max_partial_hash).map(&file)? };
    let mut buffer = vec![0; max_partial_hash];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let mut hasher = blake3::Hasher::new();
    if size >= MIN_MULTITHREADED_HASHING_SIZE {
        // hasher.update_rayon(&partial_mmap);
        hasher.update_rayon(&buffer);
    } else {
        // hasher.update(&partial_mmap);
        hasher.update(&buffer);
    }
    let hash = hasher.finalize();

    Ok(FileMetadata {
        path: file_path,
        size,
        last_modified,
        created,
        hash: hash.as_bytes().to_vec(),
    })
}
