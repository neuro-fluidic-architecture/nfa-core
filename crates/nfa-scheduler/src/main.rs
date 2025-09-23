use nfa_scheduler::{NeuroSymbolicScheduler, ResourceRequest, SchedulingPolicy, Scheduler};
use nfa_common::intent::IntentRequest;
use std::collections::HashMap;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 设置日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
    info!("Starting NFA Scheduler...");
    
    // 创建调度器
    let mut scheduler = NeuroSymbolicScheduler::new(SchedulingPolicy::PerformanceFirst);
    
    // 示例：模拟调度请求
    let intent_request = IntentRequest {
        action: "translate".to_string(),
        parameters: HashMap::from([
            ("text".to_string(), serde_json::Value::String("Hello world".to_string())),
            ("target_language".to_string(), serde_json::Value::String("zh".to_string())),
        ]),
        context: None,
    };
    
    let resource_request = ResourceRequest {
        cpu_units: 0.5,
        memory_mb: 128,
        accelerator: None,
        network_bandwidth: Some(10),
        max_latency_ms: Some(100),
    };
    
    match scheduler.schedule(&intent_request, &resource_request).await {
        Ok(allocation) => {
            info!("Resource allocated: {:?}", allocation);
        }
        Err(e) => {
            tracing::error!("Scheduling error: {}", e);
        }
    }
    
    // 等待终止信号
    signal::ctrl_c().await?;
    info!("Scheduler shutting down...");
    
    Ok(())
}