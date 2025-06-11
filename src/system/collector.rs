//! 系统信息采集器
//! 
//! 提供高级的系统信息采集接口，支持缓存和批量操作。

use crate::error::{Result, SystemMonitorError};
use crate::system::info::*;
use sysinfo::{System, ProcessRefreshKind, ProcessesToUpdate, Disks, Networks};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Notify;

/// 系统信息采集器特征
pub trait SystemInfoCollector: Send + Sync {
    /// 收集CPU信息
    fn collect_cpu_info(&self) -> Result<CpuInfo>;
    
    /// 收集内存信息
    fn collect_memory_info(&self) -> Result<MemoryInfo>;
    
    /// 收集磁盘信息
    fn collect_disk_info(&self) -> Result<Vec<DiskInfo>>;
    
    /// 收集进程信息
    fn collect_process_info(&self) -> Result<Vec<ProcessInfo>>;
    
    /// 收集系统基本信息
    fn collect_system_info(&self) -> Result<SystemInfo>;
    
    /// 收集网络信息
    fn collect_network_info(&self) -> Result<Vec<NetworkInfo>>;
    
    /// 收集完整的系统快照
    fn collect_system_snapshot(&self) -> Result<SystemSnapshot>;
}

/// 缓存的系统信息采集器
pub struct CachedSystemCollector {
    system: Arc<RwLock<System>>,
    cache: Arc<RwLock<CollectorCache>>,
    cache_duration: Duration,
    last_refresh: Arc<RwLock<Instant>>,
    refresh_notify: Arc<Notify>,
}

/// 采集器缓存
#[derive(Debug, Clone)]
struct CollectorCache {
    cpu_info: Option<(Instant, CpuInfo)>,
    memory_info: Option<(Instant, MemoryInfo)>,
    disk_info: Option<(Instant, Vec<DiskInfo>)>,
    process_info: Option<(Instant, Vec<ProcessInfo>)>,
    system_info: Option<(Instant, SystemInfo)>,
    network_info: Option<(Instant, Vec<NetworkInfo>)>,
}

impl CollectorCache {
    fn new() -> Self {
        Self {
            cpu_info: None,
            memory_info: None,
            disk_info: None,
            process_info: None,
            system_info: None,
            network_info: None,
        }
    }

    fn is_expired(&self, timestamp: Instant, cache_duration: Duration) -> bool {
        timestamp.elapsed() > cache_duration
    }
}

impl CachedSystemCollector {
    /// 创建新的缓存系统采集器
    pub fn new(cache_duration: Duration) -> Result<Self> {
        let mut system = System::new_all();
        system.refresh_all();

        Ok(Self {
            system: Arc::new(RwLock::new(system)),
            cache: Arc::new(RwLock::new(CollectorCache::new())),
            cache_duration,
            last_refresh: Arc::new(RwLock::new(Instant::now())),
            refresh_notify: Arc::new(Notify::new()),
        })
    }

    /// 强制刷新系统信息
    pub fn force_refresh(&self) -> Result<()> {
        let mut system = self.system.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统写锁".to_string()))?;
        
        system.refresh_all();
        
        // 清空缓存
        let mut cache = self.cache.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存写锁".to_string()))?;
        *cache = CollectorCache::new();
        
        // 更新刷新时间
        let mut last_refresh = self.last_refresh.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取刷新时间写锁".to_string()))?;
        *last_refresh = Instant::now();
        
        // 通知等待的任务
        self.refresh_notify.notify_waiters();
        
        Ok(())
    }

    /// 检查并刷新系统信息（如果需要）
    fn refresh_if_needed(&self) -> Result<()> {
        let last_refresh = *self.last_refresh.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取刷新时间读锁".to_string()))?;
        
        if last_refresh.elapsed() > self.cache_duration {
            self.force_refresh()?;
        }
        
        Ok(())
    }

    /// 获取或更新缓存的CPU信息
    fn get_cached_cpu_info(&self) -> Result<CpuInfo> {
        self.refresh_if_needed()?;
        
        let cache = self.cache.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存读锁".to_string()))?;
        
        if let Some((timestamp, ref info)) = cache.cpu_info {
            if !cache.is_expired(timestamp, self.cache_duration) {
                return Ok(info.clone());
            }
        }
        
        drop(cache);
        
        // 缓存过期，重新收集
        let system = self.system.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统读锁".to_string()))?;
        
        let global_cpu = system.global_cpu_usage();
        let cores: Vec<CpuCoreInfo> = system.cpus().iter().map(|cpu| CpuCoreInfo {
            name: cpu.name().to_string(),
            usage: cpu.cpu_usage(),
            frequency: cpu.frequency(),
        }).collect();

        let cpu_info = CpuInfo {
            global_usage: global_cpu,
            cores,
            core_count: system.cpus().len(),
        };
        
        drop(system);
        
        // 更新缓存
        let mut cache = self.cache.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存写锁".to_string()))?;
        cache.cpu_info = Some((Instant::now(), cpu_info.clone()));
        
        Ok(cpu_info)
    }

    /// 获取或更新缓存的内存信息
    fn get_cached_memory_info(&self) -> Result<MemoryInfo> {
        self.refresh_if_needed()?;
        
        let cache = self.cache.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存读锁".to_string()))?;
        
        if let Some((timestamp, ref info)) = cache.memory_info {
            if !cache.is_expired(timestamp, self.cache_duration) {
                return Ok(info.clone());
            }
        }
        
        drop(cache);
        
        let system = self.system.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统读锁".to_string()))?;
        
        let memory_info = MemoryInfo {
            total: system.total_memory(),
            used: system.used_memory(),
            available: system.available_memory(),
            free: system.free_memory(),
            usage_percent: (system.used_memory() as f64 / system.total_memory() as f64) * 100.0,
        };
        
        drop(system);
        
        let mut cache = self.cache.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存写锁".to_string()))?;
        cache.memory_info = Some((Instant::now(), memory_info.clone()));
        
        Ok(memory_info)
    }

    /// 获取或更新缓存的磁盘信息
    fn get_cached_disk_info(&self) -> Result<Vec<DiskInfo>> {
        self.refresh_if_needed()?;
        
        let cache = self.cache.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存读锁".to_string()))?;
        
        if let Some((timestamp, ref info)) = cache.disk_info {
            if !cache.is_expired(timestamp, self.cache_duration) {
                return Ok(info.clone());
            }
        }
        
        drop(cache);
        
        let system = self.system.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统读锁".to_string()))?;
        
        let disks: Vec<DiskInfo> = Disks::new_with_refreshed_list().iter().map(|disk| {
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
        
        drop(system);
        
        let mut cache = self.cache.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取缓存写锁".to_string()))?;
        cache.disk_info = Some((Instant::now(), disks.clone()));
        
        Ok(disks)
    }

    /// 获取系统统计信息
    pub fn get_system_stats(&self) -> Result<SystemStats> {
        let cpu_info = self.get_cached_cpu_info()?;
        let memory_info = self.get_cached_memory_info()?;
        let disk_info = self.get_cached_disk_info()?;
        
        Ok(SystemStats {
            cpu_usage_avg: cpu_info.global_usage,
            memory_usage_percent: memory_info.usage_percent,
            disk_usage_max: disk_info.iter()
                .map(|d| d.usage_percent)
                .fold(0.0, |acc, x| acc.max(x)),
            active_processes: 0, // 需要单独计算
        })
    }
}

impl SystemInfoCollector for CachedSystemCollector {
    fn collect_cpu_info(&self) -> Result<CpuInfo> {
        self.get_cached_cpu_info()
    }

    fn collect_memory_info(&self) -> Result<MemoryInfo> {
        self.get_cached_memory_info()
    }

    fn collect_disk_info(&self) -> Result<Vec<DiskInfo>> {
        self.get_cached_disk_info()
    }

    fn collect_process_info(&self) -> Result<Vec<ProcessInfo>> {
        let mut system = self.system.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统写锁".to_string()))?;
        
        system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::everything());
        
        let mut processes: Vec<ProcessInfo> = system.processes().iter().map(|(pid, process)| {
            ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                cpu_usage: process.cpu_usage(),
                memory_usage: process.memory(),
                status: format!("{:?}", process.status()),
            }
        }).collect();

        processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        processes.truncate(50);
        
        Ok(processes)
    }

    fn collect_system_info(&self) -> Result<SystemInfo> {
        let system = self.system.read()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统读锁".to_string()))?;

        Ok(SystemInfo {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
            hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
            uptime: System::uptime(),
            boot_time: System::boot_time(),
        })
    }

    fn collect_network_info(&self) -> Result<Vec<NetworkInfo>> {
        let mut system = self.system.write()
            .map_err(|_| SystemMonitorError::SystemInfo("无法获取系统写锁".to_string()))?;
        
        
        let networks: Vec<NetworkInfo> = Networks::new_with_refreshed_list().iter().map(|(name, network)| {
            NetworkInfo {
                name: name.clone(),
                bytes_received: network.received(),
                bytes_sent: network.transmitted(),
                packets_received: network.packets_received(),
                packets_sent: network.packets_transmitted(),
                errors_received: network.errors_on_received(),
                errors_sent: network.errors_on_transmitted(),
            }
        }).collect();
        
        Ok(networks)
    }

    fn collect_system_snapshot(&self) -> Result<SystemSnapshot> {
        let cpu = self.collect_cpu_info()?;
        let memory = self.collect_memory_info()?;
        let disks = self.collect_disk_info()?;
        let system = self.collect_system_info()?;
        let networks = self.collect_network_info().ok();
        
        Ok(SystemSnapshot::new(cpu, memory, disks, system, networks))
    }
}

/// 系统统计信息
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub cpu_usage_avg: f32,
    pub memory_usage_percent: f64,
    pub disk_usage_max: f64,
    pub active_processes: usize,
}

/// 批量系统信息采集器
pub struct BatchSystemCollector {
    collector: Box<dyn SystemInfoCollector>,
    batch_size: usize,
}

impl BatchSystemCollector {
    pub fn new(collector: Box<dyn SystemInfoCollector>, batch_size: usize) -> Self {
        Self {
            collector,
            batch_size,
        }
    }

    /// 批量收集系统快照
    pub async fn collect_batch_snapshots(&self, count: usize) -> Result<Vec<SystemSnapshot>> {
        let mut snapshots = Vec::with_capacity(count);
        
        for _ in 0..count {
            let snapshot = self.collector.collect_system_snapshot()?;
            snapshots.push(snapshot);
            
            if snapshots.len() % self.batch_size == 0 {
                // 批量处理间隔
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
        
        Ok(snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cached_collector_creation() {
        let collector = CachedSystemCollector::new(Duration::from_secs(1));
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_system_snapshot_collection() {
        let collector = CachedSystemCollector::new(Duration::from_secs(1)).unwrap();
        let snapshot = collector.collect_system_snapshot();
        assert!(snapshot.is_ok());
    }
}