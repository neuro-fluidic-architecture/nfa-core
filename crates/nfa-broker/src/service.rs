use crate::BrokerError;
use nfa_common::intent::{IntentRequest, IntentResponse};
use nfa_idl::IntentContract;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Request, Response, Status};

use nfa::intent::v1alpha::{
    intent_broker_server::IntentBroker, RegisterIntentRequest, RegisterIntentResponse,
    IntentMatchRequest, IntentMatchResponse, IntentContract as ProtoIntentContract,
};

#[derive(Debug, Default)]
pub struct RegisteredService {
    pub contract: IntentContract,
    pub last_heartbeat: std::time::Instant,
    pub is_healthy: bool,
}

#[derive(Debug, Default)]
pub struct BrokerService {
    services: Arc<RwLock<HashMap<String, RegisteredService>>>,
    pattern_index: Arc<RwLock<HashMap<String, Vec<String>>>>, // pattern -> service_ids
}

#[tonic::async_trait]
impl IntentBroker for BrokerService {
    async fn register_intent(
        &self,
        request: Request<RegisterIntentRequest>,
    ) -> Result<Response<RegisterIntentResponse>, Status> {
        let req = request.into_inner();
        let proto_contract = req.contract.ok_or(Status::invalid_argument("contract is required"))?;
        
        // Convert proto contract to internal representation
        let contract = self.proto_to_contract(proto_contract)?;
        
        // Validate contract
        nfa_idl::validate_contract(&contract)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        
        // Generate service ID
        let service_id = format!("{}-{}", contract.metadata.name, uuid::Uuid::new_v4());
        
        // Register service
        let mut services = self.services.write().await;
        services.insert(
            service_id.clone(),
            RegisteredService {
                contract,
                last_heartbeat: std::time::Instant::now(),
                is_healthy: true,
            },
        );
        
        // Index patterns
        self.index_patterns(&service_id, &services[&service_id].contract)
            .await;
        
        Ok(Response::new(RegisterIntentResponse {
            service_id,
            success: true,
            message: "Service registered successfully".to_string(),
        }))
    }

    async fn match_intent(
        &self,
        request: Request<IntentMatchRequest>,
    ) -> Result<Response<IntentMatchResponse>, Status> {
        let req = request.into_inner();
        let action = req.action.ok_or(Status::invalid_argument("action is required"))?;
        
        // Find matching services
        let pattern_index = self.pattern_index.read().await;
        let services = self.services.read().await;
        
        let mut matches = Vec::new();
        
        if let Some(service_ids) = pattern_index.get(&action) {
            for service_id in service_ids {
                if let Some(service) = services.get(service_id) {
                    if service.is_healthy {
                        matches.push(service_id.clone());
                    }
                }
            }
        }
        
        Ok(Response::new(IntentMatchResponse {
            service_ids: matches,
        }))
    }
}

impl BrokerService {
    pub fn new() -> Self {
        Self::default()
    }
    
    async fn index_patterns(&self, service_id: &str, contract: &IntentContract) {
        let mut pattern_index = self.pattern_index.write().await;
        
        for pattern in &contract.spec.intent_patterns {
            let action = pattern.pattern.action.clone();
            pattern_index
                .entry(action)
                .or_insert_with(Vec::new)
                .push(service_id.to_string());
        }
    }
    
    fn proto_to_contract(&self, proto: ProtoIntentContract) -> Result<IntentContract, Status> {
        // Implementation for converting proto to internal contract representation
        // This is a simplified version - actual implementation would be more complex
        Ok(IntentContract {
            version: proto.version,
            kind: proto.kind,
            metadata: nfa_idl::Metadata {
                name: proto.metadata.ok_or(Status::invalid_argument("metadata is required"))?.name,
                description: None,
                labels: None,
            },
            spec: nfa_idl::IntentSpec {
                intent_patterns: Vec::new(), // Would convert from proto
                implementation: nfa_idl::Implementation {
                    endpoint: nfa_idl::Endpoint {
                        r#type: "grpc".to_string(), // Would get from proto
                        port: Some(50051),
                        procedure: Some("test".to_string()),
                        url: None,
                    },
                    resources: None,
                },
                quality_of_service: None,
            },
        })
    }
    
    pub async fn health_check(&self) {
        // Periodically check service health
        let mut services = self.services.write().await;
        for (_, service) in services.iter_mut() {
            let elapsed = service.last_heartbeat.elapsed();
            service.is_healthy = elapsed.as_secs() < 30; // 30 second timeout
        }
    }
}