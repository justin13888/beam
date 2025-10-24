//! Simple and efficient file hashing utilities using XXH3.
//!
//! This module provides both synchronous and asynchronous APIs for computing
//! XXH3 64-bit hashes of files. The service uses a dedicated Rayon thread pool
//! configured with physical CPU cores (ignoring SMT/hyper-threading) for optimal
//! CPU-bound hashing performance.

use rayon::ThreadPool;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use crate::utils::hash::compute_hash;

/// Global dedicated thread pool for hashing operations.
///
/// This thread pool is configured to use one thread per physical CPU core,
/// ignoring SMT (simultaneous multithreading), which is optimal for CPU-bound
/// hashing workloads.
static THREAD_POOL: LazyLock<Arc<ThreadPool>> = LazyLock::new(|| {
    let num_physical_cores = num_cpus::get_physical();

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_physical_cores)
        .thread_name(|idx| format!("hash-worker-{}", idx))
        .build()
        .expect("Failed to build hash service thread pool");

    tracing::info!(
        "Initialized hash thread pool with {} threads (physical cores)",
        num_physical_cores
    );

    Arc::new(thread_pool)
});

/// Global hash service instance.
///
/// This is initialized lazily on first use and provides access to the hash service
/// which manages file hashing operations using a dedicated thread pool.
pub static HASH_SERVICE: LazyLock<Arc<HashService>> =
    LazyLock::new(|| Arc::new(HashService::new()));

/// A service that manages file hashing operations using a dedicated Rayon thread pool.
///
/// This service is designed to keep the thread pool opaque and managed internally,
/// providing a simple API for other parts of the program to use without worrying
/// about thread pool configuration or management.
#[derive(Debug)]
pub struct HashService {
    thread_pool: Arc<ThreadPool>,
}

impl HashService {
    /// Creates a new HashService that uses the global thread pool.
    fn new() -> Self {
        Self {
            thread_pool: THREAD_POOL.clone(),
        }
    }

    /// Computes the XXH3 64-bit hash of a file synchronously.
    ///
    /// This method runs the hashing operation on the service's dedicated thread pool,
    /// blocking the current thread until complete.
    pub fn hash_sync(&self, path: &Path) -> io::Result<u64> {
        let path = path.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();

        self.thread_pool.spawn(move || {
            let result = compute_hash(&path);
            let _ = tx.send(result);
        });

        rx.recv().map_err(io::Error::other)?
    }

    /// Computes the XXH3 64-bit hash of a file asynchronously.
    ///
    /// This method offloads the hashing operation to the service's dedicated thread pool,
    /// allowing the async runtime to continue processing other tasks.
    pub async fn hash_async(&self, path: PathBuf) -> io::Result<u64> {
        let thread_pool = self.thread_pool.clone();

        tokio::task::spawn_blocking(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            thread_pool.spawn(move || {
                let result = compute_hash(&path);
                let _ = tx.send(result);
            });

            rx.recv().map_err(io::Error::other)?
        })
        .await
        .map_err(io::Error::other)?
    }
}

/// Computes the XXH3 64-bit hash of a file synchronously.
///
/// This function reads the file in 1MB chunks and computes the hash using
/// a dedicated thread pool. Use this when you're already in a blocking context
/// or when you need the hash immediately and can afford to block.
///
/// # Arguments
///
/// * `path` - Path to the file to hash
///
/// # Returns
///
/// Returns a 64-bit hash value, or an IO error if the file cannot be read.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use beam_stream::services::hash;
///
/// let hash = hash::hash_file_sync(Path::new("video.mp4"))?;
/// println!("File hash: {:016x}", hash);
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn hash_file_sync(path: &Path) -> io::Result<u64> {
    HASH_SERVICE.hash_sync(path)
}

/// Computes the XXH3 64-bit hash of a file asynchronously.
///
/// This function offloads the blocking I/O operation to a dedicated thread pool,
/// allowing the async runtime to continue processing other tasks while the file
/// is being hashed. The thread pool is configured for optimal CPU-bound performance.
///
/// # Arguments
///
/// * `path` - Path to the file to hash
///
/// # Returns
///
/// Returns a 64-bit hash value, or an IO error if the file cannot be read.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use beam_stream::services::hash;
///
/// # async fn example() -> Result<(), std::io::Error> {
/// let hash = hash::hash_file(Path::new("video.mp4")).await?;
/// println!("File hash: {:016x}", hash);
/// # Ok(())
/// # }
/// ```
pub async fn hash_file(path: impl AsRef<Path>) -> io::Result<u64> {
    let path = path.as_ref().to_path_buf();
    HASH_SERVICE.hash_async(path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file_sync() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = hash_file_sync(temp_file.path()).unwrap();
        assert!(hash > 0); // Hash should be a valid u64
    }

    #[tokio::test]
    async fn test_hash_file_async() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = hash_file(temp_file.path()).await.unwrap();
        assert!(hash > 0);
    }

    #[tokio::test]
    async fn test_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Consistent data").unwrap();
        temp_file.flush().unwrap();

        let hash_sync = hash_file_sync(temp_file.path()).unwrap();
        let hash_async = hash_file(temp_file.path()).await.unwrap();

        assert_eq!(hash_sync, hash_async);
    }

    #[tokio::test]
    async fn test_concurrent_hashing() {
        // Create multiple temporary files
        let mut files = Vec::new();
        for i in 0..10 {
            let mut temp_file = NamedTempFile::new().unwrap();
            temp_file
                .write_all(format!("Test data {}", i).as_bytes())
                .unwrap();
            temp_file.flush().unwrap();
            files.push(temp_file);
        }

        // Hash all files concurrently
        let mut handles = Vec::new();
        for temp_file in &files {
            let path = temp_file.path().to_path_buf();
            let handle = tokio::spawn(async move { hash_file(&path).await });
            handles.push(handle);
        }

        // Wait for all to complete
        let results: Vec<_> = futures::future::join_all(handles).await;

        // Verify all succeeded
        for result in results {
            assert!(result.is_ok());
            let hash = result.unwrap().unwrap();
            assert!(hash > 0);
        }
    }

    #[test]
    fn test_service_initialization() {
        // Access the service to trigger initialization
        let _ = &*HASH_SERVICE;

        // The service should be initialized with physical cores
        assert!(num_cpus::get_physical() > 0);
    }
}
