//! Simple and efficient file hashing utilities using XXH3.
//!
//! This module provides both synchronous and asynchronous APIs for computing
//! XXH3 64-bit hashes of files. The service uses a dedicated Rayon thread pool
//! configured with physical CPU cores (ignoring SMT/hyper-threading) for optimal
//! CPU-bound hashing performance.

use rayon::ThreadPool;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::utils::hash::compute_hash;
#[derive(Debug, Clone)]
pub struct HashConfig {
    pub num_threads: usize,
}

impl Default for HashConfig {
    fn default() -> Self {
        Self {
            num_threads: num_cpus::get_physical(),
        }
    }
}

/// A service that manages file hashing operations.
#[async_trait::async_trait]
pub trait HashService: Send + Sync + std::fmt::Debug {
    /// Computes the XXH3 64-bit hash of a file synchronously.
    ///
    /// This method runs the hashing operation on the service's dedicated thread pool,
    /// blocking the current thread until complete.
    fn hash_sync(&self, path: &Path) -> io::Result<u64>;

    /// Computes the XXH3 64-bit hash of a file asynchronously.
    ///
    /// This method offloads the hashing operation to the service's dedicated thread pool,
    /// allowing the async runtime to continue processing other tasks.
    async fn hash_async(&self, path: PathBuf) -> io::Result<u64>;
}

/// A service that manages file hashing operations using a dedicated Rayon thread pool.
///
/// This service is designed to keep the thread pool opaque and managed internally,
/// providing a simple API for other parts of the program to use without worrying
/// about thread pool configuration or management.
#[derive(Debug, Clone)]
pub struct LocalHashService {
    thread_pool: Arc<ThreadPool>,
}

impl Default for LocalHashService {
    fn default() -> Self {
        Self::new(HashConfig::default())
    }
}

impl LocalHashService {
    /// Creates a new HashService with a dedicated thread pool.
    ///
    /// The thread pool is configured to use one thread per physical CPU core,
    /// ignoring SMT (simultaneous multithreading), which is optimal for CPU-bound
    /// hashing workloads.
    pub fn new(config: HashConfig) -> Self {
        let num_threads = if config.num_threads > 0 {
            config.num_threads
        } else {
            num_cpus::get_physical()
        };

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|idx| format!("hash-worker-{}", idx))
            .build()
            .expect("Failed to build hash service thread pool");

        tracing::info!("Initialized hash thread pool with {} threads", num_threads);

        Self {
            thread_pool: Arc::new(thread_pool),
        }
    }
}

#[async_trait::async_trait]
impl HashService for LocalHashService {
    fn hash_sync(&self, path: &Path) -> io::Result<u64> {
        let path = path.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();

        self.thread_pool.spawn(move || {
            let result = compute_hash(&path);
            let _ = tx.send(result);
        });

        rx.recv().map_err(io::Error::other)?
    }

    async fn hash_async(&self, path: PathBuf) -> io::Result<u64> {
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

        let service = LocalHashService::default();
        let hash = service.hash_sync(temp_file.path()).unwrap();
        assert!(hash > 0); // Hash should be a valid u64
    }

    #[tokio::test]
    async fn test_hash_file_async() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let service = LocalHashService::default();
        let hash = service
            .hash_async(temp_file.path().to_path_buf())
            .await
            .unwrap();
        assert!(hash > 0);
    }

    #[tokio::test]
    async fn test_hash_consistency() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Consistent data").unwrap();
        temp_file.flush().unwrap();

        let service = LocalHashService::default();
        let hash_sync = service.hash_sync(temp_file.path()).unwrap();
        let hash_async = service
            .hash_async(temp_file.path().to_path_buf())
            .await
            .unwrap();

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

        let service = LocalHashService::default();

        // Hash all files concurrently
        let mut handles = Vec::new();
        for temp_file in &files {
            let path = temp_file.path().to_path_buf();
            let service = service.clone(); // efficient clone
            let handle = tokio::spawn(async move { service.hash_async(path).await });
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
        let _ = LocalHashService::default();

        // The service should be initialized with physical cores
        assert!(num_cpus::get_physical() > 0);
    }
}
