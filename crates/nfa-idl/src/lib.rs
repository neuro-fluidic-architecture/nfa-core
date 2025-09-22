use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;
use nfa_common::intent::IntentPattern;

#[derive(Debug, Error)]
pub enum IdlError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Validation error: {0}")]
    Validation(String),
}

/// Intent Contract定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntentContract {
    pub version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: IntentSpec,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub name: String,
    pub description: Option<String>,
    pub labels: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntentSpec {
    pub intent_patterns: Vec<IntentPattern>,
    pub implementation: Implementation,
    pub quality_of_service: Option<QualityOfService>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Implementation {
    pub endpoint: Endpoint,
    pub resources: Option<Vec<ResourceRequirement>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub r#type: String, // "grpc", "http", "wasm", etc.
    pub port: Option<u32>,
    pub procedure: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceRequirement {
    pub r#type: String, // "cpu", "memory", "accelerator"
    pub units: String,  // "0.5", "128Mi", etc.
    pub kind: Option<String>, // "npu", "gpu", etc.
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QualityOfService {
    pub latency: Option<String>, // "100ms"
    pub availability: Option<String>, // "99.9%"
    pub priority: Option<String>, // "high", "medium", "low"
}

/// 从文件加载并验证Intent Contract
pub fn load_intent_contract<P: AsRef<Path>>(path: P) -> Result<IntentContract, IdlError> {
    let content = std::fs::read_to_string(path)?;
    let contract: IntentContract = serde_yaml::from_str(&content)?;
    
    // 基本验证
    if contract.version != "v1alpha" {
        return Err(IdlError::Validation("Only v1alpha version is supported".into()));
    }
    
    if contract.kind != "IntentContract" {
        return Err(IdlError::Validation("Kind must be IntentContract".into()));
    }
    
    Ok(contract)
}

/// 验证Intent Contract的完整性
pub fn validate_contract(contract: &IntentContract) -> Result<(), IdlError> {
    // 检查必需的字段
    if contract.metadata.name.is_empty() {
        return Err(IdlError::Validation("Metadata name is required".into()));
    }
    
    if contract.spec.intent_patterns.is_empty() {
        return Err(IdlError::Validation("At least one intent pattern is required".into()));
    }
    
    // 检查实现端点
    match contract.spec.implementation.endpoint.r#type.as_str() {
        "grpc" => {
            if contract.spec.implementation.endpoint.port.is_none() {
                return Err(IdlError::Validation("GRPC endpoint requires port".into()));
            }
            if contract.spec.implementation.endpoint.procedure.is_none() {
                return Err(IdlError::Validation("GRPC endpoint requires procedure".into()));
            }
        }
        "http" => {
            if contract.spec.implementation.endpoint.url.is_none() {
                return Err(IdlError::Validation("HTTP endpoint requires URL".into()));
            }
        }
        _ => return Err(IdlError::Validation("Unsupported endpoint type".into())),
    }
    
    Ok(())
}