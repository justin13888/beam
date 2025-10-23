use std::{net::{Ipv4Addr, SocketAddr}, path::PathBuf};

use serde::Deserialize;

use thiserror::Error;
use tracing::level_filters::LevelFilter;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum LogLevel {
    Error,
    Info,
    Warn,
    Debug,
    Trace,
}

impl LogLevel {
    /// Function that converts LoggingConfig to LevelFilter
    pub fn to_levelfilter(&self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

// Configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub identifier: String,
    pub production_mode: bool,
    pub log_level: LogLevel,
    pub binding_address: SocketAddr,
    pub metrics_binding_address: SocketAddr,
    pub enable_metrics: bool,
}

impl Default for Config {
    fn default() -> Self {
        let is_debug = cfg!(debug_assertions);
        
        Self {
            identifier: uuid::Uuid::new_v4().to_string(),
            production_mode: !is_debug,
            log_level: if is_debug { LogLevel::Debug } else { LogLevel::Warn },
            binding_address: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080)),
            metrics_binding_address: SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8081)),
            enable_metrics: true,
        }
    }
}

impl Config {
    pub fn with_env(env: Environment) -> Result<Self, ConfigError> {
        let mut s = Self::default();

        // Merge environment variables into config
        if let Some(production_mode) = env.production_mode {
            s.production_mode = production_mode;
        }
        if let Some(log_level) = env.log_level {
            s.log_level = log_level;
        }
        if let Some(binding_address) = env.binding_address {
            s.binding_address = binding_address;
        }
        if let Some(metrics_binding_address) = env.metrics_binding_address {
            s.metrics_binding_address = metrics_binding_address;
        }
        if let Some(enable_metrics) = env.enable_metrics {
            s.enable_metrics = enable_metrics;
        }

        Ok(s)
    }
}

// Environment variables
#[derive(Debug, Deserialize, Clone)]
pub struct Environment {
    pub production_mode: Option<bool>,
    pub log_level: Option<LogLevel>,
    pub binding_address: Option<SocketAddr>,
    pub metrics_binding_address: Option<SocketAddr>,
    pub enable_metrics: Option<bool>,
}

impl Environment {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env()
    }
}

#[derive(Debug, Error, Clone)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Environment error: {0}")]
    Env(#[from] envy::Error),
}
