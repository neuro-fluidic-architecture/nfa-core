use nfa_uhalib::{HardwareAbstraction, VirtualHardwareAbstraction};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NFA Unified Hardware Abstraction Library Demo");
    
    // 创建虚拟硬件抽象（用于演示）
    let uha = VirtualHardwareAbstraction::new()?;
    
    // 获取所有设备
    let devices = uha.get_all_devices().await?;
    println!("Discovered {} devices:", devices.len());
    
    for device in devices {
        println!("- {}: {} ({:?})", device.id, device.name, device.device_type);
    }
    
    // 获取节点资源信息
    let node_info = uha.get_node_resource_info().await?;
    println!("\nNode Resource Info:");
    println!("- Node ID: {}", node_info.node_id);
    println!("- CPU: {}/{} units available", node_info.available_cpu, node_info.total_cpu);
    println!("- Memory: {}/{} bytes available", node_info.available_memory, node_info.total_memory);
    
    if !node_info.accelerators.is_empty() {
        println!("- Accelerators:");
        for accel in node_info.accelerators {
            println!("  - {}: {}/{} units, {}/{} memory", 
                accel.kind, accel.available_units, accel.total_units,
                accel.available_memory, accel.total_memory);
        }
    }
    
    // 获取设备使用情况
    if let Some(device) = uha.get_all_devices().await?.first() {
        let usage = uha.get_device_usage(&device.id).await?;
        println!("\nDevice Usage for {}:", device.id);
        println!("- Compute: {:.1}%", usage.used_compute * 100.0);
        println!("- Memory: {} MB", usage.used_memory / 1024 / 1024);
        
        if let Some(bandwidth) = usage.used_bandwidth {
            println!("- Bandwidth: {} MB/s", bandwidth / 1024 / 1024);
        }
        
        if let Some(temp) = usage.temperature {
            println!("- Temperature: {:.1}°C", temp);
        }
        
        if let Some(power) = usage.power_usage {
            println!("- Power: {:.1}W", power);
        }
    }
    
    println!("\nDemo completed successfully!");
    Ok(())
}