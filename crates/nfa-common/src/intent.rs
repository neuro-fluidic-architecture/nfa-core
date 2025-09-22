use serde::{Deserialize, Serialize};

/// 意图模式匹配结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IntentPattern {
    pub pattern: Pattern,
    pub constraints: Option<PatternConstraints>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pattern {
    pub action: String,
    #[serde(flatten)]
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatternConstraints {
    pub required_parameters: Option<Vec<String>>,
    pub parameter_constraints: Option<std::collections::HashMap<String, ParameterConstraint>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterConstraint {
    pub r#type: Option<String>, // "string", "number", "boolean", etc.
    pub enum_values: Option<Vec<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// 意图请求和响应
#[derive(Debug, Serialize, Deserialize)]
pub struct IntentRequest {
    pub action: String,
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
    pub context: Option<IntentContext>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentContext {
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub session_id: Option<String>,
    pub preferences: Option<std::collections::HashMap<String, serde_json::Value>>,
}