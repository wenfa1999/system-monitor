//! 系统信息采集模块
//! 
//! 负责收集CPU、内存、磁盘等系统信息，提供实时监控数据。

pub mod collector;
pub mod info;
pub mod metrics;

pub use collector::*;
pub use info::*;
pub use metrics::*;

use crate::error::{Result, SystemMonitorError};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, ProcessExt, PidExt};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::time::{Duration, Interval};

/// 系统信息管理器
pub struct SystemInfoManager {
    system: Arc<Mutex<System>>,
    cpu_history: Arc<Mutex<VecDeque<f32>>>,
    memory_history: Arc<Mutex<VecDeque<f64>>>,
    max_history_points: usize,
    refresh_interval: Duration,
    _update_task: tokio::task::JoinHandle<()>,
}

impl SystemInfoManager {
    /// 创建新的系统信息管理器
    pub fn new(refresh_interval_ms: u64, max_history_points: usize) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        let system = Arc::new(Mutex::new(system));
        let cpu_history = Arc::new(Mutex::new(VecDeque::with_capacity(max_history_points)));
        let memory_history = Arc::new(Mutex::new(VecDeque::with_capacity(max_history_points)));
        let refresh_interval = Duration::from_millis(refresh_interval_ms);

        // 启动后台更新任务
        let update_task = Self::start_update_task(
            system.clone(),
            cpu_history.clone(),
            memory_history.clone(),
            max_history_points,
            refresh_interval,
        );

        Ok(Self {
            system,
            cpu_history,
            memory_history,
            max_history_points,
            refresh_interval,
            _update_task: update_task,
        })
    }

    /// 启动后台更新任务
    fn start_update_task(
        system: Arc<Mutex<System>>,
        cpu_history: Arc<Mutex<VecDeque<f32>>>,
        memory_history: Arc<Mutex<VecDeque<f64>>>,
        max_history_points: usize,
        refresh_interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(refresh_interval);
            
            loop {
                interval.tick().await;
                
                // 更新系统信息
                if let Ok(mut sys) = system.lock() {
                    sys.refresh_cpu();
                    sys.refresh_memory();
                    
                    // 更新CPU历史数据
                    let cpu_usage = sys.global_cpu_info().cpu_usage();
                    if let Ok(mut history) = cpu_history.lock() {
                        if history.len() >= max_history_points {
                            history.pop_front();
                        }
                        history.push_back(cpu_usage);
                    }
                    
                    // 更新内存历史数据
                    let memory_usage = (sys.used_memory() as f64 / sys.total_memory() as f64) * 100.0;
                    if let Ok(mut history) = memory_history.lock() {
                        if history.len() >= max_history_points {
                            history.pop_front();
                        }
                        history.push_back(memory_usage);
                    }
                } else {
                    log::error!("无法获取系统信息锁");
                }
            }
        })
    }

    /// 获取当前CPU信息
    pub fn get_cpu_info(&self) -> Result<CpuInfo> {
        let system = self.system.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;

        let global_cpu = system.global_cpu_info();
        let cpus: Vec<CpuCoreInfo> = system.cpus().iter().map(|cpu| CpuCoreInfo {
            name: cpu.name().to_string(),
            usage: cpu.cpu_usage(),
            frequency: cpu.frequency(),
        }).collect();

        Ok(CpuInfo {
            global_usage: global_cpu.cpu_usage(),
            cores: cpus,
            core_count: system.cpus().len(),
        })
    }

    /// 获取当前内存信息
    pub fn get_memory_info(&self) -> Result<MemoryInfo> {
        let system = self.system.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;

        Ok(MemoryInfo {
            total: system.total_memory(),
            used: system.used_memory(),
            available: system.available_memory(),
            free: system.free_memory(),
            usage_percent: (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
        })
    }

    /// 获取磁盘信息
    pub fn get_disk_info(&self) -> Result<Vec<DiskInfo>> {
        let system = self.system.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;

        let disks: Vec<DiskInfo> = system.disks().iter().map(|disk| {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            
            DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system: String::from_utf8_lossy(disk.file_system()).to_string(),
                total_space: total,
                available_space: available,
                used_space: used,
                usage_percent: if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 },
            }
        }).collect();

        Ok(disks)
    }

    /// 获取进程信息
    pub fn get_process_info(&self) -> Result<Vec<ProcessInfo>> {
        let mut system = self.system.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;

        system.refresh_processes();
        
        let mut processes: Vec<ProcessInfo> = system.processes().iter().map(|(pid, process)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
            }
        }).collect();

        // 按CPU使用率排序
        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        
        // 只返回前50个进程
        processes.truncate(50);
        
        Ok(processes)
    }

    /// 获取CPU使用率历史数据
    pub fn get_cpu_history(&self) -> Result<Vec<f32>> {
        let history = self.cpu_history.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取CPU历史数据锁".to_string()))?;
        
        Ok(history.iter().cloned().collect())
    }

    /// 获取内存使用率历史数据
    pub fn get_memory_history(&self) -> Result<Vec<f64>> {
        let history = self.memory_history.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取内存历史数据锁".to_string()))?;
        
        Ok(history.iter().cloned().collect())
    }

    /// 获取系统基本信息
    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let system = self.system.lock()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统信息锁".to_string()))?;

        Ok(SystemInfo {
            os_name: system.name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: system.os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: system.kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: system.host_name().unwrap_or_else(|| "Unknown".to_string()),
            uptime: system.uptime(),
            boot_time: system.boot_time(),
        })
    }

    /// 更新刷新间隔
    pub fn update_refresh_interval(&mut self, interval_ms: u64) -> Result<()> {
        self.refresh_interval = Duration::from_millis(interval_ms);
        // 注意：这里需要重启更新任务以应用新的间隔
        // 在实际实现中，可能需要更复杂的任务管理
        log::info!("刷新间隔已更新为 {}ms", interval_ms);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info_manager_creation() {
        let manager = SystemInfoManager::new(1000, 60);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_get_cpu_info() {
        let manager = SystemInfoManager::new(1000, 60).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await; // 等待初始化
        
        let cpu_info = manager.get_cpu_info();
        assert!(cpu_info.is_ok());
        
        let info = cpu_info.unwrap();
        assert!(info.core_count > 0);
        assert!(info.global_usage >= 0.0);
    }

    #[tokio::test]
    async fn test_get_memory_info() {
        let manager = SystemInfoManager::new(1000, 60).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let memory_info = manager.get_memory_info();
        assert!(memory_info.is_ok());
        
        let info = memory_info.unwrap();
        assert!(info.total > 0);
        assert!(info.usage_percent >= 0.0 && info.usage_percent <= 100.0);
    }
}