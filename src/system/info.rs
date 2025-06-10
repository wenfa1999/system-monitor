//! 系统信息数据结构定义
//! 
//! 定义了各种系统信息的数据结构，包括CPU、内存、磁盘、进程等信息。

use serde::{Deserialize, Serialize};

/// CPU信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// 全局CPU使用率
    pub global_usage: f32,
    /// CPU核心信息列表
    pub cores: Vec<CpuCoreInfo>,
    /// CPU核心数量
    pub core_count: usize,
}

/// CPU核心信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    /// 核心名称
    pub name: String,
    /// 使用率百分比
    pub usage: f32,
    /// 频率 (MHz)
    pub frequency: u64,
}

/// 内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总内存 (bytes)
    pub total: u64,
    /// 已使用内存 (bytes)
    pub used: u64,
    /// 可用内存 (bytes)
    pub available: u64,
    /// 空闲内存 (bytes)
    pub free: u64,
    /// 使用率百分比
    pub usage_percent: f64,
}

/// 磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// 磁盘名称
    pub name: String,
    /// 挂载点
    pub mount_point: String,
    /// 文件系统类型
    pub file_system: String,
    /// 总空间 (bytes)
    pub total_space: u64,
    /// 可用空间 (bytes)
    pub available_space: u64,
    /// 已使用空间 (bytes)
    pub used_space: u64,
    /// 使用率百分比
    pub usage_percent: f64,
}

/// 进程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// 进程ID
    pub pid: u32,
    /// 进程名称
    pub name: String,
    /// CPU使用率
    pub cpu_usage: f32,
    /// 内存使用量 (bytes)
    pub memory_usage: u64,
    /// 进程状态
    pub status: String,
}

/// 系统基本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// 操作系统名称
    pub os_name: String,
    /// 操作系统版本
    pub os_version: String,
    /// 内核版本
    pub kernel_version: String,
    /// 主机名
    pub hostname: String,
    /// 系统运行时间 (秒)
    pub uptime: u64,
    /// 启动时间 (Unix时间戳)
    pub boot_time: u64,
}

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// 接口名称
    pub name: String,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 发送包数
    pub packets_sent: u64,
    /// 接收错误数
    pub errors_received: u64,
    /// 发送错误数
    pub errors_sent: u64,
}

/// 系统性能快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// CPU信息
    pub cpu: CpuInfo,
    /// 内存信息
    pub memory: MemoryInfo,
    /// 磁盘信息列表
    pub disks: Vec<DiskInfo>,
    /// 系统基本信息
    pub system: SystemInfo,
    /// 网络信息列表（可选）
    pub networks: Option<Vec<NetworkInfo>>,
}

impl SystemSnapshot {
    /// 创建新的系统快照
    pub fn new(
        cpu: CpuInfo,
        memory: MemoryInfo,
        disks: Vec<DiskInfo>,
        system: SystemInfo,
        networks: Option<Vec<NetworkInfo>>,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            cpu,
            memory,
            disks,
            system,
            networks,
        }
    }

    /// 计算总体系统负载评分 (0-100)
    pub fn calculate_system_load_score(&self) -> f32 {
        let cpu_weight = 0.4;
        let memory_weight = 0.4;
        let disk_weight = 0.2;

        let cpu_score = self.cpu.global_usage;
        let memory_score = self.memory.usage_percent as f32;
        let disk_score = self.disks.iter()
            .map(|d| d.usage_percent as f32)
            .fold(0.0f32, |acc, x| acc.max(x)); // 使用最高的磁盘使用率

        cpu_score * cpu_weight + memory_score * memory_weight + disk_score as f32 * disk_weight
    }

    /// 获取系统健康状态
    pub fn get_health_status(&self) -> SystemHealthStatus {
        let load_score = self.calculate_system_load_score();
        
        match load_score {
            score if score < 30.0 => SystemHealthStatus::Excellent,
            score if score < 50.0 => SystemHealthStatus::Good,
            score if score < 70.0 => SystemHealthStatus::Fair,
            score if score < 85.0 => SystemHealthStatus::Poor,
            _ => SystemHealthStatus::Critical,
        }
    }
}

/// 系统健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemHealthStatus {
    /// 优秀 (< 30%)
    Excellent,
    /// 良好 (30-50%)
    Good,
    /// 一般 (50-70%)
    Fair,
    /// 较差 (70-85%)
    Poor,
    /// 严重 (> 85%)
    Critical,
}

impl SystemHealthStatus {
    /// 获取状态颜色 (RGB)
    pub fn color(&self) -> [f32; 3] {
        match self {
            SystemHealthStatus::Excellent => [0.2, 0.8, 0.2], // 绿色
            SystemHealthStatus::Good => [0.6, 0.8, 0.2],      // 黄绿色
            SystemHealthStatus::Fair => [0.9, 0.7, 0.2],      // 黄色
            SystemHealthStatus::Poor => [0.9, 0.5, 0.2],      // 橙色
            SystemHealthStatus::Critical => [0.9, 0.2, 0.2],  // 红色
        }
    }

    /// 获取状态描述
    pub fn description(&self) -> &'static str {
        match self {
            SystemHealthStatus::Excellent => "系统运行优秀",
            SystemHealthStatus::Good => "系统运行良好",
            SystemHealthStatus::Fair => "系统运行一般",
            SystemHealthStatus::Poor => "系统负载较高",
            SystemHealthStatus::Critical => "系统负载严重",
        }
    }
}

/// 内存单位转换工具
pub struct MemoryUnit;

impl MemoryUnit {
    /// 字节转换为人类可读格式
    pub fn bytes_to_human_readable(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: f64 = 1024.0;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// 字节转换为MB
    pub fn bytes_to_mb(bytes: u64) -> f64 {
        bytes as f64 / (1024.0 * 1024.0)
    }

    /// 字节转换为GB
    pub fn bytes_to_gb(bytes: u64) -> f64 {
        bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}

/// 时间格式化工具
pub struct TimeFormatter;

impl TimeFormatter {
    /// 将秒数转换为人类可读的时间格式
    pub fn seconds_to_human_readable(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        if days > 0 {
            format!("{}天 {}小时 {}分钟", days, hours, minutes)
        } else if hours > 0 {
            format!("{}小时 {}分钟", hours, minutes)
        } else if minutes > 0 {
            format!("{}分钟 {}秒", minutes, secs)
        } else {
            format!("{}秒", secs)
        }
    }

    /// Unix时间戳转换为本地时间字符串
    pub fn timestamp_to_local_string(timestamp: u64) -> String {
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let local_datetime = datetime.with_timezone(&chrono::Local);
        local_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_unit_conversion() {
        assert_eq!(MemoryUnit::bytes_to_human_readable(0), "0 B");
        assert_eq!(MemoryUnit::bytes_to_human_readable(1024), "1.0 KB");
        assert_eq!(MemoryUnit::bytes_to_human_readable(1048576), "1.0 MB");
        assert_eq!(MemoryUnit::bytes_to_human_readable(1073741824), "1.0 GB");
    }

    #[test]
    fn test_time_formatter() {
        assert_eq!(TimeFormatter::seconds_to_human_readable(30), "30秒");
        assert_eq!(TimeFormatter::seconds_to_human_readable(90), "1分钟 30秒");
        assert_eq!(TimeFormatter::seconds_to_human_readable(3661), "1小时 1分钟");
        assert_eq!(TimeFormatter::seconds_to_human_readable(90061), "1天 1小时 1分钟");
    }

    #[test]
    fn test_system_health_status() {
        let cpu = CpuInfo {
            global_usage: 20.0,
            cores: vec![],
            core_count: 4,
        };
        let memory = MemoryInfo {
            total: 8589934592, // 8GB
            used: 2147483648,  // 2GB
            available: 6442450944,
            free: 6442450944,
            usage_percent: 25.0,
        };
        let snapshot = SystemSnapshot::new(
            cpu,
            memory,
            vec![],
            SystemInfo {
                os_name: "Windows".to_string(),
                os_version: "11".to_string(),
                kernel_version: "10.0".to_string(),
                hostname: "test".to_string(),
                uptime: 3600,
                boot_time: 1640995200,
            },
            None,
        );

        assert_eq!(snapshot.get_health_status(), SystemHealthStatus::Excellent);
    }
}