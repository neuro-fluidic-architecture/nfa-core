use nfa_broker::Broker;
use nfa_common::errors::BrokerError;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), BrokerError> {
    // 设置日志
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    
    info!("Starting NFA Intent Broker...");
    
    // 创建并启动Broker
    let broker = Broker::new("0.0.0.0:50051").await?;
    
    // 处理优雅关机
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("failed to install CTRL+C handler");
        info!("Received shutdown signal, shutting down gracefully...");
    };
    
    // 运行Broker直到收到关机信号
    tokio::select! {
        result = broker.run() => {
            if let Err(e) = result {
                tracing::error!("Broker error: {}", e);
            }
        }
        _ = shutdown_signal => {
            info!("Shutting down broker...");
        }
    }
    
    Ok(())
}