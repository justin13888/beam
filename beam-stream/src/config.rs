use confique::Config;
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration loading error: {0}")]
    LoadError(#[from] confique::Error),

    #[error("Failed to create directory '{1}' for '{0}': {2}")]
    DirCreationError(String, String, std::io::Error),
}

/// Application configuration
#[derive(Debug, Clone, Config)]
pub struct ServerConfig {
    #[config(env = "BIND_ADDRESS", default = "0.0.0.0:8000")]
    pub bind_address: String,

    #[config(env = "SERVER_URL", default = "http://localhost:8000")]
    pub server_url: String,

    #[config(env = "ENABLE_METRICS", default = false)]
    pub enable_metrics: bool,

    #[config(env = "VIDEO_DIR", default = "./videos")]
    pub video_dir: PathBuf,

    #[config(env = "CACHE_DIR", default = "./cache")]
    pub cache_dir: PathBuf,

    #[config(
        env = "DATABASE_URL",
        default = "postgres://beam:password@localhost:5432/beam"
    )]
    pub database_url: String,

    #[config(env = "JWT_SECRET")]
    pub jwt_secret: String,

    #[config(env = "REDIS_URL", default = "redis://localhost:6379")]
    pub redis_url: String,
}

impl ServerConfig {
    /// Load configuration from environment variables and validate paths
    pub fn load_and_validate() -> Result<Self, ConfigError> {
        // 1. Load the configuration purely from environment variables
        let config = Self::builder().env().load()?;

        // 2. Perform the filesystem side-effects (creating missing parent directories)
        config.ensure_directories_exist()?;

        Ok(config)
    }

    /// Validates that the path can be created (parent directory exists or can be created)
    fn ensure_directories_exist(&self) -> Result<(), ConfigError> {
        let directories_to_check = [
            ("VIDEO_DIR", &self.video_dir),
            ("CACHE_DIR", &self.cache_dir),
        ];

        for (var_name, path) in directories_to_check {
            if let Some(parent) = path.parent()
                && !parent.exists()
            {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ConfigError::DirCreationError(
                        var_name.to_string(),
                        path.display().to_string(),
                        e,
                    )
                })?;
            }
        }

        Ok(())
    }
}
