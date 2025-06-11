//! 系统信息采集模块
//! 
//! 负责收集CPU、内存、磁盘等系统信息，提供实时监控数据。

pub mod collector;
pub mod info;
pub mod metrics;

pub use info::*;

use crate::error::{Result, SystemMonitorError};
use sysinfo::{System, Disks};
use std::sync::{Arc, Mutex};

/// 系统信息管理器
#[derive(Clone)]
pub struct SystemInfoManager {
    system: Arc<Mutex<System>>,
}

impl SystemInfoManager {
    /// 创建新的系统信息管理器
    pub fn new() -> Result<Self> {
        let system = System::new_all();
        Ok(Self {
            system: Arc::new(Mutex::new(system)),
        })
    }

    /// 异步获取系统快照
    pub async fn get_snapshot(&self) -> Result<SystemSnapshot> {
        let (cpu_info, memory_info, disk_info, system_info) = tokio::try_join!(
            self.get_cpu_info_async(),
            self.get_memory_info_async(),
            self.get_disk_info_async(),
            self.get_system_info_async()
        )?;

        Ok(SystemSnapshot::new(cpu_info, memory_info, disk_info, system_info, None))
    }

    /// 异步获取当前CPU信息
    pub async fn get_cpu_info_async(&self) -> Result<CpuInfo> {
        let system_clone = self.system.clone();
        tokio::task::spawn_blocking(move || {
            let mut system = system_clone.lock().map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;
            system.refresh_cpu_all();
            
            let global_cpu = system.global_cpu_usage();
            let cpus: Vec<CpuCoreInfo> = system.cpus().iter().map(|cpu| CpuCoreInfo {
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            }).collect();

            Ok(CpuInfo {
                global_usage: global_cpu,
                cores: cpus,
                core_count: system.cpus().len(),
            })
        }).await.map_err(|e| SystemMonitorError::Runtime(e.to_string()))?
    }

    /// 异步获取当前内存信息
    pub async fn get_memory_info_async(&self) -> Result<MemoryInfo> {
        let system_clone = self.system.clone();
        tokio::task::spawn_blocking(move || {
            let mut system = system_clone.lock().map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;
            system.refresh_memory();

            Ok(MemoryInfo {
                total: system.total_memory(),
                used: system.used_memory(),
                available: system.available_memory(),
                free: system.free_memory(),
                usage_percent: (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
            })
        }).await.map_err(|e| SystemMonitorError::Runtime(e.to_string()))?
    }

    /// 异步获取磁盘信息
    pub async fn get_disk_info_async(&self) -> Result<Vec<DiskInfo>> {
        tokio::task::spawn_blocking(move || {
            let disks = Disks::new_with_refreshed_list();
            let disk_info: Vec<DiskInfo> = disks.iter().map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total - available;
                
                DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    file_system: String::from_utf8_lossy(disk.file_system().as_encoded_bytes()).to_string(),
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    usage_percent: if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 },
                }
            }).collect();
            Ok(disk_info)
        }).await.map_err(|e| SystemMonitorError::Runtime(e.to_string()))?
    }

    /// 异步获取系统基本信息
    pub async fn get_system_info_async(&self) -> Result<SystemInfo> {
        tokio::task::spawn_blocking(move || {
            Ok(SystemInfo {
                os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
                os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
                kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
                hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
                uptime: System::uptime(),
                boot_time: System::boot_time(),
            })
        }).await.map_err(|e| SystemMonitorError::Runtime(e.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info_manager_creation() {
        let manager = SystemInfoManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_get_cpu_info_async() {
        let manager = SystemInfoManager::new().unwrap();
        
        let cpu_info = manager.get_cpu_info_async().await;
        assert!(cpu_info.is_ok());
        
        let info = cpu_info.unwrap();
        assert!(info.core_count > 0);
        // Note: global_usage might be 0.0 on the first call.
    }

    #[tokio::test]
    async fn test_get_memory_info_async() {
        let manager = SystemInfoManager::new().unwrap();
        
        let memory_info = manager.get_memory_info_async().await;
        assert!(memory_info.is_ok());
        
        let info = memory_info.unwrap();
        assert!(info.total > 0);
        assert!(info.usage_percent >= 0.0 && info.usage_percent <= 100.0);
    }
}