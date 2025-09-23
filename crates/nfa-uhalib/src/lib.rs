use async_trait::async_trait;
use nfa_common::types::{AcceleratorInfo, NodeResourceInfo};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UHAError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    
    #[error("Device access denied: {0}")]
    AccessDenied(String),
    
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    
    #[error("Driver error: {0}")]
    DriverError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// 统一硬件抽象 trait
#[async_trait]
pub trait HardwareAbstraction: Send + Sync {
    /// 获取所有设备信息
    async fn get_all_devices(&self) -> Result<Vec<DeviceInfo>, UHAError>;
    
    /// 按类型获取设备
    async fn get_devices_by_type(&self, device_type: DeviceType) -> Result<Vec<DeviceInfo>, UHAError>;
    
    /// 获取设备资源使用情况
    async fn get_device_usage(&self, device_id: &str) -> Result<DeviceUsage, UHAError>;
    
    /// 分配设备资源
    async fn allocate_device(
        &self,
        device_id: &str,
        resource_request: &ResourceRequest,
    ) -> Result<AllocationHandle, UHAError>;
    
    /// 释放设备资源
    async fn release_device(&self, handle: AllocationHandle) -> Result<(), UHAError>;
    
    /// 获取节点资源信息
    async fn get_node_resource_info(&self) -> Result<NodeResourceInfo, UHAError>;
}

/// 设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Cpu,
    Gpu,
    Tpu,
    Npu,
    Fpga,
    Memory,
    Network,
    Storage,
}

/// 设备信息
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub vendor: String,
    pub model: String,
    pub capabilities: HashMap<String, String>,
    pub total_resources: DeviceResources,
    pub available_resources: DeviceResources,
}

/// 设备资源
#[derive(Debug, Clone)]
pub struct DeviceResources {
    pub compute_units: f64,
    pub memory_bytes: u64,
    pub bandwidth: Option<u64>,
    pub specialized_units: Option<f64>,
}

/// 设备使用情况
#[derive(Debug, Clone)]
pub struct DeviceUsage {
    pub device_id: String,
    pub used_compute: f64,
    pub used_memory: u64,
    pub used_bandwidth: Option<u64>,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
}

/// 资源请求
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub compute_units: f64,
    pub memory_bytes: u64,
    pub bandwidth: Option<u64>,
    pub specialized_units: Option<f64>,
    pub timeout_ms: Option<u64>,
}

/// 分配句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AllocationHandle(uuid::Uuid);

impl AllocationHandle {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for AllocationHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// 统一硬件抽象实现
pub struct UnifiedHardwareAbstraction {
    devices: HashMap<String, DeviceInfo>,
    allocations: HashMap<AllocationHandle, DeviceAllocation>,
}

impl UnifiedHardwareAbstraction {
    pub fn new() -> Result<Self, UHAError> {
        let mut uha = Self {
            devices: HashMap::new(),
            allocations: HashMap::new(),
        };
        
        // 扫描并初始化设备
        uha.scan_devices()?;
        
        Ok(uha)
    }
    
    fn scan_devices(&mut self) -> Result<(), UHAError> {
        // 这里简化实现，实际中会调用平台特定的设备发现
        
        // 添加CPU设备
        self.devices.insert(
            "cpu-0".to_string(),
            DeviceInfo {
                id: "cpu-0".to_string(),
                name: "Main CPU".to_string(),
                device_type: DeviceType::Cpu,
                vendor: "Generic".to_string(),
                model: "x86_64".to_string(),
                capabilities: HashMap::from([
                    ("architecture".to_string(), "x86_64".to_string()),
                    ("cores".to_string(), "8".to_string()),
                    ("threads".to_string(), "16".to_string()),
                ]),
                total_resources: DeviceResources {
                    compute_units: 16.0, // 16线程
                    memory_bytes: 0, // CPU不直接管理内存
                    bandwidth: None,
                    specialized_units: None,
                },
                available_resources: DeviceResources {
                    compute_units: 16.0,
                    memory_bytes: 0,
                    bandwidth: None,
                    specialized_units: None,
                },
            },
        );
        
        // 添加GPU设备（如果可用）
        if cfg!(feature = "nvidia") {
            if let Ok(gpu_info) = self.scan_nvidia_gpus() {
                for gpu in gpu_info {
                    self.devices.insert(gpu.id.clone(), gpu);
                }
            }
        }
        
        Ok(())
    }
    
    #[cfg(feature = "nvidia")]
    fn scan_nvidia_gpus(&self) -> Result<Vec<DeviceInfo>, UHAError> {
        use nvml_wrapper::NVML;
        
        let mut gpus = Vec::new();
        
        match NVML::init() {
            Ok(nvml) => {
                if let Ok(count) = nvml.device_count() {
                    for i in 0..count {
                        if let Ok(device) = nvml.device_by_index(i) {
                            if let (Ok(name), Ok(memory), Ok(uuid)) = (
                                device.name(),
                                device.memory_info(),
                                device.uuid(),
                            ) {
                                let total_memory = memory.total;
                                
                                let gpu = DeviceInfo {
                                    id: format!("gpu-{}", i),
                                    name,
                                    device_type: DeviceType::Gpu,
                                    vendor: "NVIDIA".to_string(),
                                    model: "GPU".to_string(),
                                    capabilities: HashMap::from([
                                        ("cuda_cores".to_string(), "0".to_string()), // 需要实际获取
                                        ("tensor_cores".to_string(), "0".to_string()), // 需要实际获取
                                    ]),
                                    total_resources: DeviceResources {
                                        compute_units: 1.0, // 简化表示
                                        memory_bytes: total_memory,
                                        bandwidth: None,
                                        specialized_units: None,
                                    },
                                    available_resources: DeviceResources {
                                        compute_units: 1.0,
                                        memory_bytes: total_memory,
                                        bandwidth: None,
                                        specialized_units: None,
                                    },
                                };
                                
                                gpus.push(gpu);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to initialize NVML: {}", e);
            }
        }
        
        Ok(gpus)
    }
    
    #[cfg(not(feature = "nvidia"))]
    fn scan_nvidia_gpus(&self) -> Result<Vec<DeviceInfo>, UHAError> {
        Ok(Vec::new())
    }
}

#[async_trait]
impl HardwareAbstraction for UnifiedHardwareAbstraction {
    async fn get_all_devices(&self) -> Result<Vec<DeviceInfo>, UHAError> {
        Ok(self.devices.values().cloned().collect())
    }
    
    async fn get_devices_by_type(&self, device_type: DeviceType) -> Result<Vec<DeviceInfo>, UHAError> {
        Ok(self
            .devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect())
    }
    
    async fn get_device_usage(&self, device_id: &str) -> Result<DeviceUsage, UHAError> {
        let device = self
            .devices
            .get(device_id)
            .ok_or_else(|| UHAError::DeviceNotFound(device_id.to_string()))?;
        
        // 简化实现，实际中会查询设备实际使用情况
        Ok(DeviceUsage {
            device_id: device_id.to_string(),
            used_compute: 0.0,
            used_memory: 0,
            used_bandwidth: None,
            temperature: None,
            power_usage: None,
        })
    }
    
    async fn allocate_device(
        &self,
        device_id: &str,
        resource_request: &ResourceRequest,
    ) -> Result<AllocationHandle, UHAError> {
        let _device = self
            .devices
            .get(device_id)
            .ok_or_else(|| UHAError::DeviceNotFound(device_id.to_string()))?;
        
        // 检查资源是否足够（简化实现）
        // 实际中会检查设备的可用资源
        
        let handle = AllocationHandle::new();
        
        Ok(handle)
    }
    
    async fn release_device(&self, _handle: AllocationHandle) -> Result<(), UHAError> {
        // 释放资源（简化实现）
        Ok(())
    }
    
    async fn get_node_resource_info(&self) -> Result<NodeResourceInfo, UHAError> {
        // 获取系统总资源信息（简化实现）
        Ok(NodeResourceInfo {
            node_id: "local-node".to_string(),
            total_cpu: 16.0,
            available_cpu: 16.0,
            total_memory: 16 * 1024 * 1024 * 1024, // 16GB
            available_memory: 16 * 1024 * 1024 * 1024,
            accelerators: Vec::new(),
            network_bandwidth: 1000, // 1Gbps
            network_latency: 1,      // 1ms
            location: None,
        })
    }
}

/// 平台特定的硬件抽象
#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    
    /// Linux特定的硬件抽象实现
    pub struct LinuxHardwareAbstraction {
        udev: Option<libudev::Context>,
    }
    
    impl LinuxHardwareAbstraction {
        pub fn new() -> Result<Self, UHAError> {
            let udev = libudev::Context::new().ok();
            
            Ok(Self { udev })
        }
        
        pub fn scan_udev_devices(&self) -> Result<Vec<DeviceInfo>, UHAError> {
            let mut devices = Vec::new();
            
            if let Some(context) = &self.udev {
                let mut enumerator = libudev::Enumerator::new(context).map_err(|e| {
                    UHAError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                })?;
                
                // 扫描GPU设备
                enumerator
                    .match_subsystem("drm")
                    .map_err(|e| UHAError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
                
                for device in enumerator.scan_devices().map_err(|e| {
                    UHAError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                })? {
                    if let (Some(sysname), Some(devtype)) = (device.sysname(), device.devtype()) {
                        if sysname.contains("card") && devtype == "drm_minor" {
                            let vendor = device
                                .property_value("ID_VENDOR_FROM_DATABASE")
                                .and_then(|v| v.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            let model = device
                                .property_value("ID_MODEL_FROM_DATABASE")
                                .and_then(|v| v.to_str())
                                .unwrap_or("Unknown")
                                .to_string();
                            
                            let gpu = DeviceInfo {
                                id: sysname.to_string_lossy().to_string(),
                                name: format!("{} {}", vendor, model),
                                device_type: DeviceType::Gpu,
                                vendor,
                                model,
                                capabilities: HashMap::new(),
                                total_resources: DeviceResources {
                                    compute_units: 1.0,
                                    memory_bytes: 0, // 需要实际查询
                                    bandwidth: None,
                                    specialized_units: None,
                                },
                                available_resources: DeviceResources {
                                    compute_units: 1.0,
                                    memory_bytes: 0,
                                    bandwidth: None,
                                    specialized_units: None,
                                },
                            };
                            
                            devices.push(gpu);
                        }
                    }
                }
            }
            
            Ok(devices)
        }
    }
}

/// 虚拟硬件抽象（用于测试和开发）
pub struct VirtualHardwareAbstraction {
    devices: HashMap<String, DeviceInfo>,
}

impl VirtualHardwareAbstraction {
    pub fn new() -> Result<Self, UHAError> {
        let mut devices = HashMap::new();
        
        // 添加虚拟CPU
        devices.insert(
            "virtual-cpu-0".to_string(),
            DeviceInfo {
                id: "virtual-cpu-0".to_string(),
                name: "Virtual CPU".to_string(),
                device_type: DeviceType::Cpu,
                vendor: "NFA".to_string(),
                model: "vCPU".to_string(),
                capabilities: HashMap::from([
                    ("architecture".to_string(), "virtual".to_string()),
                    ("cores".to_string(), "8".to_string()),
                    ("threads".to_string(), "16".to_string()),
                ]),
                total_resources: DeviceResources {
                    compute_units: 16.0,
                    memory_bytes: 0,
                    bandwidth: None,
                    specialized_units: None,
                },
                available_resources: DeviceResources {
                    compute_units: 16.0,
                    memory_bytes: 0,
                    bandwidth: None,
                    specialized_units: None,
                },
            },
        );
        
        // 添加虚拟GPU
        devices.insert(
            "virtual-gpu-0".to_string(),
            DeviceInfo {
                id: "virtual-gpu-0".to_string(),
                name: "Virtual GPU".to_string(),
                device_type: DeviceType::Gpu,
                vendor: "NFA".to_string(),
                model: "vGPU".to_string(),
                capabilities: HashMap::from([
                    ("cuda_cores".to_string(), "1024".to_string()),
                    ("tensor_cores".to_string(), "128".to_string()),
                ]),
                total_resources: DeviceResources {
                    compute_units: 1.0,
                    memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
                    bandwidth: Some(100 * 1024 * 1024),   // 100MB/s
                    specialized_units: Some(1.0),
                },
                available_resources: DeviceResources {
                    compute_units: 1.0,
                    memory_bytes: 8 * 1024 * 1024 * 1024,
                    bandwidth: Some(100 * 1024 * 1024),
                    specialized_units: Some(1.0),
                },
            },
        );
        
        Ok(Self { devices })
    }
}

#[async_trait]
impl HardwareAbstraction for VirtualHardwareAbstraction {
    async fn get_all_devices(&self) -> Result<Vec<DeviceInfo>, UHAError> {
        Ok(self.devices.values().cloned().collect())
    }
    
    async fn get_devices_by_type(&self, device_type: DeviceType) -> Result<Vec<DeviceInfo>, UHAError> {
        Ok(self
            .devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect())
    }
    
    async fn get_device_usage(&self, device_id: &str) -> Result<DeviceUsage, UHAError> {
        let _device = self
            .devices
            .get(device_id)
            .ok_or_else(|| UHAError::DeviceNotFound(device_id.to_string()))?;
        
        Ok(DeviceUsage {
            device_id: device_id.to_string(),
            used_compute: 0.3, // 30%使用率
            used_memory: 512 * 1024 * 1024, // 512MB
            used_bandwidth: Some(10 * 1024 * 1024), // 10MB/s
            temperature: Some(45.0), // 45°C
            power_usage: Some(75.0), // 75W
        })
    }
    
    async fn allocate_device(
        &self,
        device_id: &str,
        resource_request: &ResourceRequest,
    ) -> Result<AllocationHandle, UHAError> {
        let _device = self
            .devices
            .get(device_id)
            .ok_or_else(|| UHAError::DeviceNotFound(device_id.to_string()))?;
        
        // 虚拟实现，总是成功
        let handle = AllocationHandle::new();
        Ok(handle)
    }
    
    async fn release_device(&self, _handle: AllocationHandle) -> Result<(), UHAError> {
        // 虚拟实现，总是成功
        Ok(())
    }
    
    async fn get_node_resource_info(&self) -> Result<NodeResourceInfo, UHAError> {
        Ok(NodeResourceInfo {
            node_id: "virtual-node".to_string(),
            total_cpu: 16.0,
            available_cpu: 12.8, // 80%可用
            total_memory: 16 * 1024 * 1024 * 1024, // 16GB
            available_memory: 12 * 1024 * 1024 * 1024, // 12GB可用
            accelerators: vec![AcceleratorInfo {
                kind: "vGPU".to_string(),
                total_units: 1.0,
                available_units: 0.8, // 80%可用
                total_memory: 8 * 1024 * 1024 * 1024, // 8GB
                available_memory: 6 * 1024 * 1024 * 1024, // 6GB可用
            }],
            network_bandwidth: 1000, // 1Gbps
            network_latency: 5,      // 5ms
            location: None,
        })
    }
}