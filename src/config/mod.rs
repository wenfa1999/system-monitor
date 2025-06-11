//! 配置管理模块
//! 
//! 负责应用程序配置的加载、保存和管理，支持用户自定义设置。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{Result, SystemMonitorError};

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct AppConfig {
    /// 窗口配置
    pub window: WindowConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
    /// UI配置
    pub ui: UiConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// 窗口宽度
    pub width: f32,
    /// 窗口高度
    pub height: f32,
    /// 是否最大化
    pub maximized: bool,
    /// 是否置顶
    pub always_on_top: bool,
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 刷新间隔（毫秒）
    pub refresh_interval_ms: u64,
    /// 是否启用CPU监控
    pub enable_cpu_monitoring: bool,
    /// 是否启用内存监控
    pub enable_memory_monitoring: bool,
    /// 是否启用磁盘监控
    pub enable_disk_monitoring: bool,
    /// 是否启用进程监控
    pub enable_process_monitoring: bool,
    /// CPU历史数据点数量
    pub cpu_history_points: usize,
    /// 内存历史数据点数量
    pub memory_history_points: usize,
}

/// UI配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// 主题
    pub theme: Theme,
    /// 字体大小
    pub font_size: f32,
    /// 是否显示网格
    pub show_grid: bool,
    /// 图表颜色配置
    pub chart_colors: ChartColors,
    /// 默认标签页
    pub default_tab: String,
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    Auto,
}

/// 图表颜色配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartColors {
    /// CPU使用率颜色
    pub cpu_color: [f32; 3],
    /// 内存使用率颜色
    pub memory_color: [f32; 3],
    /// 磁盘使用率颜色
    pub disk_color: [f32; 3],
    /// 网格颜色
    pub grid_color: [f32; 3],
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 是否启用缓存
    pub enable_caching: bool,
    /// 缓存大小（MB）
    pub cache_size_mb: usize,
    /// 是否启用多线程
    pub enable_multithreading: bool,
    /// 工作线程数量
    pub worker_threads: usize,
}


impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 800.0,
            maximized: false,
            always_on_top: false,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            refresh_interval_ms: 1000,
            enable_cpu_monitoring: true,
            enable_memory_monitoring: true,
            enable_disk_monitoring: true,
            enable_process_monitoring: false,
            cpu_history_points: 60,
            memory_history_points: 60,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Auto,
            font_size: 14.0,
            show_grid: true,
            chart_colors: ChartColors::default(),
            default_tab: "overview".to_string(),
        }
    }
}

impl Default for ChartColors {
    fn default() -> Self {
        Self {
            cpu_color: [0.2, 0.7, 0.9],      // 蓝色
            memory_color: [0.9, 0.6, 0.2],   // 橙色
            disk_color: [0.3, 0.8, 0.3],     // 绿色
            grid_color: [0.5, 0.5, 0.5],     // 灰色
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_size_mb: 64,
            enable_multithreading: true,
            worker_threads: num_cpus::get().min(4),
        }
    }
}

impl AppConfig {
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if !config_path.exists() {
            log::info!("配置文件不存在，创建默认配置: {:?}", config_path);
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| SystemMonitorError::Config(format!("读取配置文件失败: {}", e)))?;

        let config: Self = serde_json::from_str(&config_str)
            .map_err(|e| SystemMonitorError::Config(format!("解析配置文件失败: {}", e)))?;

        log::info!("成功加载配置文件: {:?}", config_path);
        Ok(config)
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // 确保配置目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SystemMonitorError::Config(format!("创建配置目录失败: {}", e)))?;
        }

        let config_str = serde_json::to_string_pretty(self)
            .map_err(|e| SystemMonitorError::Config(format!("序列化配置失败: {}", e)))?;

        std::fs::write(&config_path, config_str)
            .map_err(|e| SystemMonitorError::Config(format!("写入配置文件失败: {}", e)))?;

        log::info!("配置文件已保存: {:?}", config_path);
        Ok(())
    }

    /// 获取配置文件路径
    fn get_config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| SystemMonitorError::Config("无法获取配置目录".to_string()))?;
        
        Ok(config_dir.join("system-monitor").join("config.json"))
    }

    /// 验证配置有效性
    pub fn validate(&self) -> Result<()> {
        // 验证刷新间隔
        if self.monitoring.refresh_interval_ms < 100 || self.monitoring.refresh_interval_ms > 10000 {
            return Err(SystemMonitorError::Config(
                "刷新间隔必须在100-10000毫秒之间".to_string()
            ));
        }

        // 验证历史数据点数量
        if self.monitoring.cpu_history_points == 0 || self.monitoring.cpu_history_points > 1000 {
            return Err(SystemMonitorError::Config(
                "CPU历史数据点数量必须在1-1000之间".to_string()
            ));
        }

        // 验证窗口尺寸
        if self.window.width < 800.0 || self.window.height < 600.0 {
            return Err(SystemMonitorError::Config(
                "窗口尺寸不能小于800x600".to_string()
            ));
        }

        // 验证字体大小
        if self.ui.font_size < 8.0 || self.ui.font_size > 32.0 {
            return Err(SystemMonitorError::Config(
                "字体大小必须在8-32之间".to_string()
            ));
        }

        Ok(())
    }

    /// 重置为默认配置
    pub fn reset_to_default(&mut self) {
        *self = Self::default();
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: AppConfig,
    auto_save: bool,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new(auto_save: bool) -> Result<Self> {
        let config = AppConfig::load()?;
        config.validate()?;
        
        Ok(Self { config, auto_save })
    }

    /// 获取配置引用
    pub fn get(&self) -> &AppConfig {
        &self.config
    }

    /// 获取可变配置引用
    pub fn get_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// 更新配置
    pub fn update<F>(&mut self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);
        self.config.validate()?;
        
        if self.auto_save {
            self.config.save()?;
        }
        
        Ok(())
    }

    /// 手动保存配置
    pub fn save(&self) -> Result<()> {
        self.config.save()
    }
}

// 添加num_cpus依赖到Cargo.toml中需要的功能

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.window.width, 1200.0);
        assert_eq!(config.monitoring.refresh_interval_ms, 1000);
        assert!(config.monitoring.enable_cpu_monitoring);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.monitoring.refresh_interval_ms = 50; // 太小
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.window.width, deserialized.window.width);
    }
}