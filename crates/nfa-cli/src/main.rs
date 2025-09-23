use clap::{Parser, Subcommand};
use nfa_broker::BrokerClient;
use nfa_idl::{load_intent_contract, validate_contract};
use std::path::PathBuf;
use tonic::transport::Channel;

#[derive(Parser)]
#[command(name = "nfa")]
#[command(about = "NFA Command Line Interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register an intent service
    Register {
        /// Path to the intent contract YAML file
        #[arg(short, long)]
        contract: PathBuf,
        
        /// Broker address
        #[arg(short, long, default_value = "http://localhost:50051")]
        broker: String,
    },
    
    /// List registered services
    List {
        /// Broker address
        #[arg(short, long, default_value = "http://localhost:50051")]
        broker: String,
    },
    
    /// Match an intent
    Match {
        /// Intent action
        #[arg(short, long)]
        action: String,
        
        /// Intent parameters (key=value format)
        #[arg(short, long)]
        params: Vec<String>,
        
        /// Broker address
        #[arg(short, long, default_value = "http://localhost:50051")]
        broker: String,
    },
    
    /// Validate an intent contract
    Validate {
        /// Path to the intent contract YAML file
        #[arg(short, long)]
        contract: PathBuf,
    },
    
    /// Start a local development node
    Dev {
        /// Port for the local broker
        #[arg(short, long, default_value_t = 50051)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Register { contract, broker } => {
            register_command(contract, broker).await?;
        }
        Commands::List { broker } => {
            list_command(broker).await?;
        }
        Commands::Match { action, params, broker } => {
            match_command(action, params, broker).await?;
        }
        Commands::Validate { contract } => {
            validate_command(contract).await?;
        }
        Commands::Dev { port } => {
            dev_command(port).await?;
        }
    }
    
    Ok(())
}

async fn register_command(contract_path: PathBuf, broker_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Registering intent service from: {:?}", contract_path);
    
    // 加载和验证契约
    let contract = load_intent_contract(&contract_path)?;
    validate_contract(&contract)?;
    
    println!("Contract validated successfully: {}", contract.metadata.name);
    
    // 连接到Broker
    let mut client = BrokerClient::connect(broker_addr).await?;
    
    // 注册服务
    let response = client.register_intent(contract).await?;
    println!("Service registered successfully with ID: {}", response.service_id);
    
    Ok(())
}

async fn list_command(broker_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Listing registered services from: {}", broker_addr);
    
    // 连接到Broker
    let mut client = BrokerClient::connect(broker_addr).await?;
    
    // 获取服务列表
    let services = client.list_services().await?;
    
    if services.is_empty() {
        println!("No services registered.");
    } else {
        println!("Registered services:");
        for service in services {
            println!("- {}: {}", service.id, service.name);
        }
    }
    
    Ok(())
}

async fn match_command(
    action: String,
    params: Vec<String>,
    broker_addr: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Matching intent: {}", action);
    
    // 解析参数
    let mut parameters = std::collections::HashMap::new();
    for param in params {
        let parts: Vec<&str> = param.splitn(2, '=').collect();
        if parts.len() == 2 {
            parameters.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    
    // 连接到Broker
    let mut client = BrokerClient::connect(broker_addr).await?;
    
    // 匹配意图
    let matches = client.match_intent(&action, parameters).await?;
    
    if matches.is_empty() {
        println!("No matching services found.");
    } else {
        println!("Matching services:");
        for service_id in matches {
            println!("- {}", service_id);
        }
    }
    
    Ok(())
}

async fn validate_command(contract_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating intent contract: {:?}", contract_path);
    
    // 加载和验证契约
    let contract = load_intent_contract(&contract_path)?;
    validate_contract(&contract)?;
    
    println!("Contract is valid!");
    println!("Name: {}", contract.metadata.name);
    println!("Description: {:?}", contract.metadata.description);
    println!("Patterns: {}", contract.spec.intent_patterns.len());
    
    for pattern in &contract.spec.intent_patterns {
        println!("- Action: {}", pattern.pattern.action);
    }
    
    Ok(())
}

async fn dev_command(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting local development node on port {}", port);
    
    // 启动本地Broker
    let broker_handle = tokio::spawn(async move {
        let broker = nfa_broker::Broker::new(&format!("0.0.0.0:{}", port))
            .await
            .expect("Failed to create broker");
        broker.run().await.expect("Broker failed");
    });
    
    println!("Local broker started. Press Ctrl+C to stop.");
    
    // 等待终止信号
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    
    // 停止Broker
    broker_handle.abort();
    
    Ok(())
}