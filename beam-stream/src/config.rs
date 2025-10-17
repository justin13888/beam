use std::env;
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error type
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid boolean value '{1}' for environment variable '{0}'")]
    InvalidBoolValue(String, String),
    #[error("Invalid path '{1}' for environment variable '{0}'")]
    InvalidPath(String, String),
    #[error("Failed to parse value '{1}' for environment variable '{0}'")]
    ParseError(String, String),
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub server_url: String,
    pub enable_metrics: bool,
    pub video_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        let server_url =
            env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        let enable_metrics = parse_bool_env("ENABLE_METRICS", false)?;

        let video_dir = parse_path_env("VIDEO_DIR", "./videos")?;
        let cache_dir = parse_path_env("CACHE_DIR", "./cache")?;

        Ok(Config {
            bind_address,
            server_url,
            enable_metrics,
            video_dir,
            cache_dir,
        })
    }
}

/// Parse a boolean environment variable with a default value
fn parse_bool_env(var_name: &str, default: bool) -> Result<bool, ConfigError> {
    match env::var(var_name) {
        Ok(value) => match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(ConfigError::InvalidBoolValue(var_name.to_string(), value)),
        },
        Err(_) => Ok(default),
    }
}

/// Parse a path environment variable with a default value
fn parse_path_env(var_name: &str, default: &str) -> Result<PathBuf, ConfigError> {
    let path_str = env::var(var_name).unwrap_or_else(|_| default.to_string());

    let path = PathBuf::from(&path_str);

    // Validate that the path can be created (parent directory exists or can be created)
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            // Try to create the parent directory to validate the path
            if let Err(_) = std::fs::create_dir_all(parent) {
                return Err(ConfigError::InvalidPath(var_name.to_string(), path_str));
            }
        }
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_bool_env() {
        // Test true values
        unsafe { env::set_var("TEST_BOOL_TRUE", "true") };
        assert!(parse_bool_env("TEST_BOOL_TRUE", false).unwrap());

        unsafe { env::set_var("TEST_BOOL_1", "1") };
        assert!(parse_bool_env("TEST_BOOL_1", false).unwrap());

        unsafe { env::set_var("TEST_BOOL_YES", "yes") };
        assert!(parse_bool_env("TEST_BOOL_YES", false).unwrap());

        // Test false values
        unsafe { env::set_var("TEST_BOOL_FALSE", "false") };
        assert!(!parse_bool_env("TEST_BOOL_FALSE", true).unwrap());

        unsafe { env::set_var("TEST_BOOL_0", "0") };
        assert!(!parse_bool_env("TEST_BOOL_0", true).unwrap());

        // Test default when not set
        unsafe { env::remove_var("TEST_BOOL_UNSET") };
        assert!(parse_bool_env("TEST_BOOL_UNSET", true).unwrap());

        // Test invalid value
        unsafe { env::set_var("TEST_BOOL_INVALID", "maybe") };
        assert!(parse_bool_env("TEST_BOOL_INVALID", false).is_err());

        // Cleanup
        unsafe {
            env::remove_var("TEST_BOOL_TRUE");
            env::remove_var("TEST_BOOL_1");
            env::remove_var("TEST_BOOL_YES");
            env::remove_var("TEST_BOOL_FALSE");
            env::remove_var("TEST_BOOL_0");
            env::remove_var("TEST_BOOL_INVALID");
        }
    }

    #[test]
    fn test_config_from_env() {
        // Test with default values
        unsafe {
            env::remove_var("BIND_ADDRESS");
            env::remove_var("ENABLE_METRICS");
            env::remove_var("VIDEO_DIR");
            env::remove_var("CACHE_DIR");
        }

        let config = Config::from_env().unwrap();
        assert_eq!(config.bind_address, "0.0.0.0:3000");
        assert!(!config.enable_metrics);
        assert_eq!(config.video_dir, PathBuf::from("./videos"));
        assert_eq!(config.cache_dir, PathBuf::from("./cache"));

        // Test with custom values
        unsafe {
            env::set_var("BIND_ADDRESS", "127.0.0.1:8080");
            env::set_var("ENABLE_METRICS", "true");
            env::set_var("VIDEO_DIR", "/tmp/videos");
            env::set_var("CACHE_DIR", "/tmp/cache");
        }

        let config = Config::from_env().unwrap();
        assert_eq!(config.bind_address, "127.0.0.1:8080");
        assert_eq!(config.enable_metrics, true);
        assert_eq!(config.video_dir, PathBuf::from("/tmp/videos"));
        assert_eq!(config.cache_dir, PathBuf::from("/tmp/cache"));

        // Cleanup
        unsafe {
            env::remove_var("BIND_ADDRESS");
            env::remove_var("ENABLE_METRICS");
            env::remove_var("VIDEO_DIR");
            env::remove_var("CACHE_DIR");
        }
    }
}
