# Rust+egui系统监控工具 - 实施指导文档

## 1. 实施准备

### 1.1 开发环境配置

```bash
# 安装Rust工具链
rustup install stable
rustup default stable
rustup component add rustfmt clippy

# 安装开发工具
cargo install cargo-watch
cargo install cargo-audit
cargo install cargo-tarpaulin  # 代码覆盖率工具

# 创建项目
cargo new system-monitor --bin
cd system-monitor
```

### 1.2 初始Cargo.toml配置

```toml
[package]
name = "system-monitor"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A system monitoring tool built with Rust and egui"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/system-monitor"
keywords = ["system", "monitor", "gui", "windows"]
categories = ["gui", "system-tools"]

[dependencies]
# GUI框架
egui = "0.24"
eframe = { version = "0.24", default-features = false, features = [
    "default_fonts",
    "glow",
    "persistence",
] }

# 系统信息采集
sysinfo = "0.29"

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# 序列化
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 时间处理
chrono = { version = "0.4", features = ["serde"] }

# 错误处理
thiserror = "1.0"
anyhow = "1.0"

# 日志
log = "0.4"
env_logger = "0.10"

# 数据结构
smallvec = { version = "1.11", features = ["serde"] }
lru = "0.12"

# Windows特定
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winuser",
    "processthreadsapi",
    "psapi",
    "sysinfoapi",
] }

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
mockall = "0.11"
tempfile = "3.8"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 0
debug = true

[[bench]]
name = "data_collection"
harness = false

[[bench]]
name = "ui_rendering"
harness = false
```

## 2. 核心模块实施指导

### 2.1 项目结构创建

```bash
# 创建目录结构
mkdir -p src/{app,ui/{components,widgets,themes},business,data/{collectors,processors,models,history},infrastructure,platform/{windows,traits},utils,config}
mkdir -p tests/{integration,unit}
mkdir -p benches
mkdir -p assets/{icons,configs}
mkdir -p docs
mkdir -p scripts

# 创建模块文件
touch src/lib.rs
touch src/app/{mod.rs,application.rs,ui_controller.rs,event_handler.rs}
touch src/ui/{mod.rs,components/mod.rs,widgets/mod.rs,themes/mod.rs}
touch src/business/{mod.rs,data_manager.rs,config_manager.rs,state_manager.rs,notification_manager.rs}
touch src/data/{mod.rs,collectors/mod.rs,processors/mod.rs,models/mod.rs,history/mod.rs}
touch src/infrastructure/{mod.rs,timer_service.rs,logging_service.rs,error_handler.rs,performance_monitor.rs}
touch src/platform/{mod.rs,windows/mod.rs,traits.rs}
touch src/utils/{mod.rs,formatters.rs,validators.rs,helpers.rs}
touch src/config/{mod.rs,app_config.rs,ui_config.rs,default_config.rs}
```

### 2.2 核心接口实现

#### src/lib.rs
```rust
//! System Monitor - A Rust+egui based system monitoring tool
//! 
//! This crate provides a comprehensive system monitoring solution
//! with real-time data collection and visualization.

pub mod app;
pub mod ui;
pub mod business;
pub mod data;
pub mod infrastructure;
pub mod platform;
pub mod utils;
pub mod config;

// Re-export commonly used types
pub use app::Application;
pub use config::AppConfig;
pub use data::models::*;
pub use infrastructure::error_handler::SystemMonitorError;

/// Application result type
pub type Result<T> = std::result::Result<T, SystemMonitorError>;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
```

#### src/platform/traits.rs
```rust
//! Platform abstraction traits

use async_trait::async_trait;
use std::time::Duration;
use crate::data::models::*;
use crate::Result;

/// System information collection trait
#[async_trait]
pub trait SystemInfoCollector: Send + Sync {
    /// Collect CPU information
    async fn collect_cpu_info(&self) -> Result<CpuInfo>;
    
    /// Collect memory information
    async fn collect_memory_info(&self) -> Result<MemoryInfo>;
    
    /// Collect disk information
    async fn collect_disk_info(&self) -> Result<Vec<DiskInfo>>;
    
    /// Collect system information
    async fn collect_system_info(&self) -> Result<SystemInfo>;
    
    /// Set collection interval
    fn set_collection_interval(&mut self, interval: Duration);
    
    /// Get current collection interval
    fn get_collection_interval(&self) -> Duration;
}

/// Data processing trait
pub trait DataProcessor<T>: Send + Sync {
    type Input;
    type Output;
    
    /// Process input data
    fn process(&self, input: Self::Input) -> Result<Self::Output>;
    
    /// Validate input data
    fn validate(&self, data: &Self::Input) -> Result<()>;
    
    /// Format output data for display
    fn format(&self, data: &Self::Output) -> String;
}

/// UI component trait
pub trait UiComponent {
    type Data;
    type Config;
    
    /// Render the component
    fn render(&mut self, ui: &mut egui::Ui, data: &Self::Data, config: &Self::Config);
    
    /// Handle user interactions
    fn handle_interaction(&mut self, response: &egui::Response) -> Option<UiEvent>;
    
    /// Update component configuration
    fn update_config(&mut self, config: Self::Config);
}

/// UI events
#[derive(Debug, Clone)]
pub enum UiEvent {
    TabChanged(String),
    SettingsChanged,
    RefreshRequested,
    ExitRequested,
}
```

### 2.3 数据模型定义

#### src/data/models/mod.rs
```rust
//! Data models for system monitoring

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub mod cpu_data;
pub mod memory_data;
pub mod disk_data;
pub mod system_data;

pub use cpu_data::*;
pub use memory_data::*;
pub use disk_data::*;
pub use system_data::*;

/// Complete system data snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub disk_info: Vec<DiskInfo>,
    pub system_info: SystemInfo,
    pub timestamp: DateTime<Utc>,
}

impl SystemSnapshot {
    pub fn new(
        cpu_info: CpuInfo,
        memory_info: MemoryInfo,
        disk_info: Vec<DiskInfo>,
        system_info: SystemInfo,
    ) -> Self {
        Self {
            cpu_info,
            memory_info,
            disk_info,
            system_info,
            timestamp: Utc::now(),
        }
    }
}
```

#### src/data/models/cpu_data.rs
```rust
//! CPU data models

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// Overall CPU usage percentage (0.0 - 100.0)
    pub usage_percent: f32,
    
    /// Per-core usage percentages
    pub core_usage: SmallVec<[f32; 16]>,
    
    /// CPU frequency in MHz
    pub frequency_mhz: u64,
    
    /// CPU temperature in Celsius (if available)
    pub temperature_celsius: Option<f32>,
    
    /// Number of logical cores
    pub logical_cores: usize,
    
    /// Number of physical cores
    pub physical_cores: usize,
    
    /// CPU brand/model name
    pub brand: String,
}

impl CpuInfo {
    /// Get the maximum core usage
    pub fn max_core_usage(&self) -> f32 {
        self.core_usage.iter().fold(0.0, |max, &usage| max.max(usage))
    }
    
    /// Get the minimum core usage
    pub fn min_core_usage(&self) -> f32 {
        self.core_usage.iter().fold(100.0, |min, &usage| min.min(usage))
    }
    
    /// Check if CPU usage is high (>80%)
    pub fn is_high_usage(&self) -> bool {
        self.usage_percent > 80.0
    }
    
    /// Get usage level as enum
    pub fn usage_level(&self) -> UsageLevel {
        match self.usage_percent {
            x if x < 30.0 => UsageLevel::Low,
            x if x < 70.0 => UsageLevel::Medium,
            _ => UsageLevel::High,
        }
    }
}

/// CPU usage level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsageLevel {
    Low,
    Medium,
    High,
}

impl UsageLevel {
    /// Get color for usage level
    pub fn color(&self) -> egui::Color32 {
        match self {
            UsageLevel::Low => egui::Color32::from_rgb(16, 185, 129),    // Green
            UsageLevel::Medium => egui::Color32::from_rgb(245, 158, 11), // Yellow
            UsageLevel::High => egui::Color32::from_rgb(239, 68, 68),    // Red
        }
    }
}
```

### 2.4 错误处理实现

#### src/infrastructure/error_handler.rs
```rust
//! Error handling for the system monitor

use thiserror::Error;

/// Main error type for the system monitor
#[derive(Debug, Error)]
pub enum SystemMonitorError {
    #[error("Data collection error: {0}")]
    Collection(#[from] CollectionError),
    
    #[error("Data processing error: {0}")]
    Processing(#[from] ProcessingError),
    
    #[error("UI rendering error: {0}")]
    Rendering(#[from] RenderingError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("System error: {0}")]
    System(#[from] SystemError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Data collection errors
#[derive(Debug, Error)]
pub enum CollectionError {
    #[error("System API call failed: {message}")]
    ApiCallFailed { message: String },
    
    #[error("Insufficient permissions to access system information")]
    InsufficientPermissions,
    
    #[error("Operation timed out after {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
    
    #[error("Invalid data format received from system")]
    InvalidData,
    
    #[error("System resource unavailable: {resource}")]
    ResourceUnavailable { resource: String },
}

/// Data processing errors
#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("Data validation failed: {reason}")]
    ValidationFailed { reason: String },
    
    #[error("Calculation error: {operation}")]
    CalculationError { operation: String },
    
    #[error("Data format conversion failed")]
    ConversionFailed,
}

/// UI rendering errors
#[derive(Debug, Error)]
pub enum RenderingError {
    #[error("Widget rendering failed: {widget}")]
    WidgetFailed { widget: String },
    
    #[error("Layout calculation failed")]
    LayoutFailed,
    
    #[error("Resource loading failed: {resource}")]
    ResourceLoadFailed { resource: String },
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Invalid configuration value: {key} = {value}")]
    InvalidValue { key: String, value: String },
    
    #[error("Configuration parsing failed")]
    ParseFailed,
    
    #[error("Configuration save failed")]
    SaveFailed,
}

/// System-level errors
#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Platform not supported: {platform}")]
    UnsupportedPlatform { platform: String },
    
    #[error("Required system feature unavailable: {feature}")]
    FeatureUnavailable { feature: String },
    
    #[error("System resource exhausted: {resource}")]
    ResourceExhausted { resource: String },
}

/// Error recovery actions
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    /// Retry the operation with the given policy
    Retry(RetryPolicy),
    /// Use fallback data/behavior
    UseFallback,
    /// Log the error and continue
    LogAndContinue,
    /// Shutdown the application
    Shutdown,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

/// Error recovery manager
pub struct ErrorRecoveryManager {
    retry_policies: std::collections::HashMap<String, RetryPolicy>,
}

impl ErrorRecoveryManager {
    pub fn new() -> Self {
        let mut manager = Self {
            retry_policies: std::collections::HashMap::new(),
        };
        
        // Configure default retry policies
        manager.retry_policies.insert(
            "data_collection".to_string(),