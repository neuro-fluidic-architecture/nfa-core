use nfa_common::intent::{IntentRequest, IntentResponse};
use nfa_idl::IntentContract;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{transport::Server, Request, Response, Status};

mod service;
pub use service::BrokerService;

pub struct Broker {
    address: String,
    service: BrokerService,
}

impl Broker {
    pub async fn new(address: &str) -> Result<Self, BrokerError> {
        let service = BrokerService::default();
        Ok(Self {
            address: address.to_string(),
            service,
        })
    }
    
    pub async fn run(self) -> Result<(), BrokerError> {
        let addr = self.address.parse().expect("invalid address");
        let service = self.service;
        
        info!("Broker listening on {}", addr);
        
        Server::builder()
            .add_service(BrokerService::server(service))
            .serve(addr)
            .await
            .map_err(|e| BrokerError::ServerError(e.to_string()))?;
            
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BrokerError {
    #[error("Server error: {0}")]
    ServerError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    // 其他错误变体...
}