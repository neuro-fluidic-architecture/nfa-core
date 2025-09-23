use tonic::transport::Channel;
use tonic::Request;
use nfa_common::intent::{IntentRequest, IntentResponse};
use nfa_idl::IntentContract;
use thiserror::Error;

use nfa::broker::v1alpha::{
    intent_broker_client::IntentBrokerClient,
    RegisterIntentRequest, RegisterIntentResponse,
    IntentMatchRequest, IntentMatchResponse,
    HeartbeatRequest, HeartbeatResponse,
    UnregisterIntentRequest, UnregisterIntentResponse,
};
use nfa::intent::v1alpha::{IntentPattern, IntentContext};

#[derive(Debug, Error)]
pub enum BrokerClientError {
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),
    
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Broker客户端
pub struct BrokerClient {
    client: IntentBrokerClient<Channel>,
}

impl BrokerClient {
    /// 连接到Broker
    pub async fn connect(addr: String) -> Result<Self, BrokerClientError> {
        let client = IntentBrokerClient::connect(addr).await?;
        Ok(Self { client })
    }
    
    /// 注册意图服务
    pub async fn register_intent(
        &mut self,
        contract: IntentContract,
    ) -> Result<RegisterIntentResponse, BrokerClientError> {
        // 转换为protobuf格式
        let proto_contract = self.contract_to_proto(contract)?;
        
        let request = Request::new(RegisterIntentRequest {
            contract: Some(proto_contract),
        });
        
        let response = self.client.register_intent(request).await?;
        Ok(response.into_inner())
    }
    
    /// 匹配意图
    pub async fn match_intent(
        &mut self,
        pattern: IntentPattern,
        context: Option<IntentContext>,
    ) -> Result<IntentMatchResponse, BrokerClientError> {
        let proto_pattern = self.pattern_to_proto(pattern)?;
        let proto_context = context.map(|c| self.context_to_proto(c));
        
        let request = Request::new(IntentMatchRequest {
            pattern: Some(proto_pattern),
            context: proto_context,
        });
        
        let response = self.client.match_intent(request).await?;
        Ok(response.into_inner())
    }
    
    /// 发送心跳
    pub async fn heartbeat(
        &mut self,
        service_id: String,
    ) -> Result<HeartbeatResponse, BrokerClientError> {
        let request = Request::new(HeartbeatRequest { service_id });
        
        let response = self.client.heartbeat(request).await?;
        Ok(response.into_inner())
    }
    
    /// 取消注册服务
    pub async fn unregister_intent(
        &mut self,
        service_id: String,
    ) -> Result<UnregisterIntentResponse, BrokerClientError> {
        let request = Request::new(UnregisterIntentRequest { service_id });
        
        let response = self.client.unregister_intent(request).await?;
        Ok(response.into_inner())
    }
    
    /// 将内部契约转换为protobuf格式
    fn contract_to_proto(&self, contract: IntentContract) -> Result<nfa::intent::v1alpha::IntentContract, BrokerClientError> {
        // 简化实现，实际中需要完整转换
        Ok(nfa::intent::v1alpha::IntentContract {
            version: contract.version,
            kind: contract.kind,
            metadata: Some(nfa::intent::v1alpha::Metadata {
                name: contract.metadata.name,
                description: contract.metadata.description.unwrap_or_default(),
                labels: contract.metadata.label.unwrap_or_default(),
            }),
            spec: Some(nfa::intent::v1alpha::IntentSpec {
                intent_patterns: vec![], // 需要完整转换
                implementation: Some(nfa::intent::v1alpha::Implementation {
                    endpoint: Some(nfa::intent::v1alpha::Endpoint {
                        r#type: "grpc".to_string(),
                        address: Some(nfa::intent::v1alpha::endpoint::Address::Grpc(
                            nfa::intent::v1alpha::GrpcAddress {
                                port: 50051,
                                procedure: "test".to_string(),
                            }
                        )),
                    }),
                    resources: vec![],
                }),
                quality_of_service: Some(nfa::intent::v1alpha::QualityOfService {
                    latency: "100ms".to_string(),
                    availability: "99.9%".to_string(),
                    priority: "high".to_string(),
                }),
            }),
        })
    }
    
    /// 将内部模式转换为protobuf格式
    fn pattern_to_proto(&self, pattern: nfa_common::intent::IntentPattern) -> Result<nfa::intent::v1alpha::IntentPattern, BrokerClientError> {
        // 简化实现
        Ok(nfa::intent::v1alpha::IntentPattern {
            pattern: Some(nfa::intent::v1alpha::intent_pattern::Pattern {
                action: pattern.pattern.action,
                parameters: std::collections::HashMap::new(), // 需要完整转换
            }),
            constraints: None,
        })
    }
    
    /// 将内部上下文转换为protobuf格式
    fn context_to_proto(&self, context: nfa_common::intent::IntentContext) -> nfa::intent::v1alpha::IntentContext {
        nfa::intent::v1alpha::IntentContext {
            user_id: context.user_id.unwrap_or_default(),
            device_id: context.device_id.unwrap_or_default(),
            session_id: context.session_id.unwrap_or_default(),
            preferences: std::collections::HashMap::new(), // 需要完整转换
        }
    }
}