use async_trait::async_trait;
use nfa_idl::IntentContract;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),
    
    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),
    
    #[error("Database error: {0}")]
    Database(String),
}

/// 存储后端 trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 存储服务契约
    async fn store_service(&self, service_id: String, contract: IntentContract) -> Result<(), StorageError>;
    
    /// 获取服务契约
    async fn get_service(&self, service_id: &str) -> Result<Option<IntentContract>, StorageError>;
    
    /// 删除服务
    async fn delete_service(&self, service_id: &str) -> Result<(), StorageError>;
    
    /// 获取所有服务ID
    async fn get_all_service_ids(&self) -> Result<Vec<String>, StorageError>;
    
    /// 按意图模式查找服务
    async fn find_services_by_pattern(&self, pattern: &str) -> Result<Vec<String>, StorageError>;
}

/// 内存存储后端（用于开发和测试）
pub struct MemoryStorage {
    services: Arc<RwLock<HashMap<String, IntentContract>>>,
    pattern_index: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            pattern_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for MemoryStorage {
    async fn store_service(&self, service_id: String, contract: IntentContract) -> Result<(), StorageError> {
        let mut services = self.services.write().await;
        if services.contains_key(&service_id) {
            return Err(StorageError::ServiceAlreadyExists(service_id));
        }
        
        // 存储服务
        services.insert(service_id.clone(), contract.clone());
        
        // 更新模式索引
        let mut pattern_index = self.pattern_index.write().await;
        for intent_pattern in &contract.spec.intent_patterns {
            let action = intent_pattern.pattern.action.clone();
            pattern_index
                .entry(action)
                .or_insert_with(Vec::new)
                .push(service_id.clone());
        }
        
        Ok(())
    }
    
    async fn get_service(&self, service_id: &str) -> Result<Option<IntentContract>, StorageError> {
        let services = self.services.read().await;
        Ok(services.get(service_id).cloned())
    }
    
    async fn delete_service(&self, service_id: &str) -> Result<(), StorageError> {
        let mut services = self.services.write().await;
        
        // 获取服务以从索引中移除
        if let Some(contract) = services.get(service_id) {
            let mut pattern_index = self.pattern_index.write().await;
            
            // 从模式索引中移除
            for intent_pattern in &contract.spec.intent_patterns {
                let action = intent_pattern.pattern.action.clone();
                if let Some(service_ids) = pattern_index.get_mut(&action) {
                    service_ids.retain(|id| id != service_id);
                    
                    // 如果没有服务匹配此模式，移除条目
                    if service_ids.is_empty() {
                        pattern_index.remove(&action);
                    }
                }
            }
        }
        
        // 从服务存储中移除
        services.remove(service_id);
        
        Ok(())
    }
    
    async fn get_all_service_ids(&self) -> Result<Vec<String>, StorageError> {
        let services = self.services.read().await;
        Ok(services.keys().cloned().collect())
    }
    
    async fn find_services_by_pattern(&self, pattern: &str) -> Result<Vec<String>, StorageError> {
        let pattern_index = self.pattern_index.read().await;
        Ok(pattern_index.get(pattern).cloned().unwrap_or_default())
    }
}

/// Redis存储后端（用于生产环境）
pub struct RedisStorage {
    client: redis::Client,
    prefix: String,
}

impl RedisStorage {
    pub fn new(url: &str, prefix: &str) -> Result<Self, StorageError> {
        let client = redis::Client::open(url)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }
    
    fn service_key(&self, service_id: &str) -> String {
        format!("{}:services:{}", self.prefix, service_id)
    }
    
    fn pattern_key(&self, pattern: &str) -> String {
        format!("{}:patterns:{}", self.prefix, pattern)
    }
}

#[async_trait]
impl StorageBackend for RedisStorage {
    async fn store_service(&self, service_id: String, contract: IntentContract) -> Result<(), StorageError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // 检查服务是否已存在
        let key = self.service_key(&service_id);
        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        if exists {
            return Err(StorageError::ServiceAlreadyExists(service_id));
        }
        
        // 序列化契约
        let contract_data = serde_json::to_string(&contract)
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // 存储服务
        redis::cmd("SET")
            .arg(&key)
            .arg(&contract_data)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // 更新模式索引
        for intent_pattern in &contract.spec.intent_patterns {
            let pattern_key = self.pattern_key(&intent_pattern.pattern.action);
            redis::cmd("SADD")
                .arg(&pattern_key)
                .arg(&service_id)
                .query_async(&mut conn).await
                .map_err(|e| StorageError::Database(e.to_string()))?;
        }
        
        Ok(())
    }
    
    async fn get_service(&self, service_id: &str) -> Result<Option<IntentContract>, StorageError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let key = self.service_key(service_id);
        let data: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        match data {
            Some(data) => {
                let contract: IntentContract = serde_json::from_str(&data)
                    .map_err(|e| StorageError::Database(e.to_string()))?;
                Ok(Some(contract))
            }
            None => Ok(None),
        }
    }
    
    async fn delete_service(&self, service_id: &str) -> Result<(), StorageError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // 获取服务以从索引中移除
        if let Some(contract) = self.get_service(service_id).await? {
            // 从模式索引中移除
            for intent_pattern in &contract.spec.intent_patterns {
                let pattern_key = self.pattern_key(&intent_pattern.pattern.action);
                redis::cmd("SREM")
                    .arg(&pattern_key)
                    .arg(service_id)
                    .query_async(&mut conn).await
                    .map_err(|e| StorageError::Database(e.to_string()))?;
            }
        }
        
        // 从服务存储中移除
        let key = self.service_key(service_id);
        redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    async fn get_all_service_ids(&self) -> Result<Vec<String>, StorageError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let pattern = format!("{}:services:*", self.prefix);
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        // 从键中提取服务ID
        let service_ids = keys.iter()
            .filter_map(|key| key.split(':').last().map(|s| s.to_string()))
            .collect();
        
        Ok(service_ids)
    }
    
    async fn find_services_by_pattern(&self, pattern: &str) -> Result<Vec<String>, StorageError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        let pattern_key = self.pattern_key(pattern);
        let service_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(&pattern_key)
            .query_async(&mut conn).await
            .map_err(|e| StorageError::Database(e.to_string()))?;
        
        Ok(service_ids)
    }
}