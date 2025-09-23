use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 节点资源信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResourceInfo {
    pub node_id: String,
    pub total_cpu: f64,
    pub available_cpu: f64,
    pub total_memory: u64,
    pub available_memory: u64,
    pub accelerators: Vec<AcceleratorInfo>,
    pub network_bandwidth: u64,
    pub network_latency: u64,
    pub location: Option<NodeLocation>,
}

/// 加速器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceleratorInfo {
    pub kind: String,
    pub total_units: f64,
    pub available_units: f64,
    pub total_memory: u64,
    pub available_memory: u64,
}

/// 节点位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLocation {
    pub region: String,
    pub zone: String,
    pub datacenter: String,
    pub coordinates: Option<Coordinates>,
}

/// 地理坐标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

/// 服务质量指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityOfServiceMetrics {
    pub latency_ms: u64,
    pub throughput: f64,
    pub availability: f64,
    pub error_rate: f64,
    pub cost_per_request: f64,
}

/// 服务健康状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ServiceHealth {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

/// 服务状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub service_id: String,
    pub health: ServiceHealth,
    pub last_heartbeat: u64, // Unix timestamp
    pub metrics: QualityOfServiceMetrics,
    pub load: f64, // 0.0 to 1.0
}

/// 系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub broker: BrokerConfig,
    pub scheduler: SchedulerConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
}

/// Broker配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerConfig {
    pub listen_address: String,
    pub max_connections: u32,
    pub heartbeat_timeout_secs: u64,
    pub storage_backend: StorageBackendType,
}

/// 存储后端类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    Memory,
    Redis { url: String, prefix: String },
    Postgres { url: String, table_prefix: String },
}

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub policy: SchedulingPolicy,
    pub resource_check_interval_secs: u64,
    pub max_concurrent_schedules: u32,
    pub cost_weight: f64,
    pub latency_weight: f64,
    pub energy_weight: f64,
}

/// 调度策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulingPolicy {
    PerformanceFirst,
    EnergyEfficient,
    LatencySensitive,
    CostOptimized,
    Balanced,
}

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub max_retention_days: u32,
    pub backup_interval_secs: u64,
    pub cleanup_interval_secs: u64,
}

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub timeout_secs: u64,
    pub retry_attempts: u32,
    pub retry_delay_secs: u64,
    pub max_message_size: u32,
}