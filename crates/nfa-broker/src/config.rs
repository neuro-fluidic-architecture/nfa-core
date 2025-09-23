use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

use nfa_common::types::{BrokerConfig, StorageBackendType};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    
    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

/// Broker配置
#[derive(Debug, Clone, Deserialize)]
pub struct BrokerConfig {
    pub listen_address: String,
    pub max_connections: u32,
    pub heartbeat_timeout_secs: u64,
    pub storage_backend: StorageBackendConfig,
}

/// 存储后端配置
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StorageBackendConfig {
    Memory,
    Redis { url: String, prefix: String },
    Postgres { url: String, table_prefix: String },
}

/// 加载配置
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<BrokerConfig, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config: BrokerConfig = toml::from_str(&content)?;
    
    // 验证配置
    validate_config(&config)?;
    
    Ok(config)
}

/// 从环境变量加载配置
pub fn load_config_from_env() -> Result<BrokerConfig, ConfigError> {
    let config = BrokerConfig {
        listen_address: std::env::var("NFA_BROKER_LISTEN_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:50051".to_string()),
        max_connections: std::env::var("NFA_BROKER_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .map_err(|e| ConfigError::Invalid(format!("Invalid max_connections: {}", e)))?,
        heartbeat_timeout_secs: std::env::var("NFA_BROKER_HEARTBEAT_TIMEOUT")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .map_err(|e| ConfigError::Invalid(format!("Invalid heartbeat_timeout_secs: {}", e)))?,
        storage_backend: load_storage_backend_from_env()?,
    };
    
    validate_config(&config)?;
    Ok(config)
}

/// 从环境变量加载存储后端配置
fn load_storage_backend_from_env() -> Result<StorageBackendConfig, ConfigError> {
    let backend_type = std::env::var("NFA_STORAGE_BACKEND")
        .unwrap_or_else(|_| "memory".to_string());
    
    match backend_type.as_str() {
        "memory" => Ok(StorageBackendConfig::Memory),
        "redis" => {
            let url = std::env::var("NFA_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());
            let prefix = std::env::var("NFA_REDIS_PREFIX")
                .unwrap_or_else(|_| "nfa".to_string());
            
            Ok(StorageBackendConfig::Redis { url, prefix })
        }
        "postgres" => {
            let url = std::env::var("NFA_POSTGRES_URL")
                .unwrap_or_else(|_| "postgres://user:password@localhost:5432/nfa".to_string());
            let table_prefix = std::env::var("NFA_POSTGRES_TABLE_PREFIX")
                .unwrap_or_else(|_| "nfa_".to_string());
            
            Ok(StorageBackendConfig::Postgres { url, table_prefix })
        }
        _ => Err(ConfigError::Invalid(format!("Unknown storage backend: {}", backend_type))),
    }
}

/// 验证配置
fn validate_config(config: &BrokerConfig) -> Result<(), ConfigError> {
    // 验证监听地址
    if config.listen_address.is_empty() {
        return Err(ConfigError::Invalid("listen_address cannot be empty".to_string()));
    }
    
    // 验证最大连接数
    if config.max_connections == 0 {
        return Err(ConfigError::Invalid("max_connections must be greater than 0".to_string()));
    }
    
    // 验证心跳超时
    if config.heartbeat_timeout_secs == 0 {
        return Err(ConfigError::Invalid("heartbeat_timeout_secs must be greater than 0".to_string()));
    }
    
    Ok(())
}

/// 默认配置
impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:50051".to_string(),
            max_connections: 1000,
            heartbeat_timeout_secs: 30,
            storage_backend: StorageBackendConfig::Memory,
        }
    }
}