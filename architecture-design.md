# Rust+egui系统监控工具 - 架构设计

## 项目概述

基于Rust和egui构建的Windows系统监控工具，提供实时系统性能信息显示。

## 系统架构图

```mermaid
graph TB
    subgraph "应用层"
        A[主应用程序 main.rs]
        B[应用状态管理 app.rs]
        C[配置管理 config.rs]
    end
    
    subgraph "界面层"
        D[主界面 ui/main_window.rs]
        E[CPU监控面板 ui/cpu_panel.rs]
        F[内存监控面板 ui/memory_panel.rs]
        G[磁盘监控面板 ui/disk_panel.rs]
        H[系统信息面板 ui/system_panel.rs]
    end
    
    subgraph "数据层"
        I[系统信息采集器 data/collector.rs]
        J[CPU数据处理 data/cpu_data.rs]
        K[内存数据处理 data/memory_data.rs]
        L[磁盘数据处理 data/disk_data.rs]
        M[历史数据管理 data/history.rs]
    end
    
    subgraph "工具层"
        N[图表渲染 utils/charts.rs]
        O[数据格式化 utils/formatter.rs]
        P[定时器管理 utils/timer.rs]
    end
    
    subgraph "外部依赖"
        Q[sysinfo - 系统信息]
        R[egui - GUI框架]
        S[eframe - 应用框架]
    end
    
    A --> B
    A --> C
    B --> D
    D --> E
    D --> F
    D --> G
    D --> H
    
    E --> J
    F --> K
    G --> L
    H --> I
    
    I --> Q
    J --> I
    K --> I
    L --> I
    
    E --> N
    F --> N
    G --> N
    
    J --> O
    K --> O
    L --> O
    
    B --> P
    M --> J
    M --> K
    M --> L
    
    D --> R
    A --> S
```

## 核心模块设计

### 1. 主应用程序 (main.rs)
```rust
// 应用程序入口点
// 初始化egui应用
// 设置窗口属性和图标
```

### 2. 应用状态管理 (app.rs)
```rust
pub struct SystemMonitorApp {
    // 系统数据采集器
    collector: SystemCollector,
    // 历史数据存储
    history: DataHistory,
    // 应用配置
    config: AppConfig,
    // UI状态
    ui_state: UiState,
    // 更新定时器
    update_timer: Timer,
}
```

### 3. 系统信息采集 (data/collector.rs)
```rust
pub struct SystemCollector {
    system: System,
    last_update: Instant,
    update_interval: Duration,
}

impl SystemCollector {
    pub fn new() -> Self
    pub fn update(&mut self) -> Result<SystemData, CollectorError>
    pub fn get_cpu_usage(&self) -> Vec<f32>
    pub fn get_memory_info(&self) -> MemoryInfo
    pub fn get_disk_info(&self) -> Vec<DiskInfo>
}
```

### 4. 界面组件设计

#### CPU监控面板 (ui/cpu_panel.rs)
```rust
pub struct CpuPanel {
    history: VecDeque<f32>,
    max_history_size: usize,
}

impl CpuPanel {
    pub fn show(&mut self, ui: &mut Ui, cpu_data: &CpuData)
    fn draw_cpu_chart(&self, ui: &mut Ui, data: &[f32])
    fn draw_cpu_cores(&self, ui: &mut Ui, cores: &[f32])
}
```

#### 内存监控面板 (ui/memory_panel.rs)
```rust
pub struct MemoryPanel;

impl MemoryPanel {
    pub fn show(&mut self, ui: &mut Ui, memory_data: &MemoryData)
    fn draw_memory_usage_bar(&self, ui: &mut Ui, used: u64, total: u64)
    fn draw_memory_details(&self, ui: &mut Ui, data: &MemoryData)
}
```

## 数据流设计

```mermaid
sequenceDiagram
    participant App as 主应用
    participant Collector as 数据采集器
    participant UI as 用户界面
    participant History as 历史数据
    
    App->>Collector: 定时更新请求
    Collector->>Collector: 采集系统信息
    Collector->>History: 存储历史数据
    Collector->>App: 返回最新数据
    App->>UI: 更新界面数据
    UI->>UI: 重绘界面组件
```

## 项目结构

```
system-monitor/
├── Cargo.toml
├── src/
│   ├── main.rs                 # 应用入口
│   ├── app.rs                  # 主应用逻辑
│   ├── config.rs               # 配置管理
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs      # 主窗口
│   │   ├── cpu_panel.rs        # CPU监控面板
│   │   ├── memory_panel.rs     # 内存监控面板
│   │   ├── disk_panel.rs       # 磁盘监控面板
│   │   └── system_panel.rs     # 系统信息面板
│   ├── data/
│   │   ├── mod.rs
│   │   ├── collector.rs        # 系统信息采集
│   │   ├── cpu_data.rs         # CPU数据处理
│   │   ├── memory_data.rs      # 内存数据处理
│   │   ├── disk_data.rs        # 磁盘数据处理
│   │   └── history.rs          # 历史数据管理
│   └── utils/
│       ├── mod.rs
│       ├── charts.rs           # 图表渲染工具
│       ├── formatter.rs        # 数据格式化
│       └── timer.rs            # 定时器管理
├── assets/
│   └── icon.ico                # 应用图标
└── README.md
```

## 核心依赖关系

```toml
[dependencies]
egui = "0.24"
eframe = { version = "0.24", default-features = false, features = [
    "default_fonts",
    "glow",
    "persistence",
] }
sysinfo = "0.29"
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
```

## 性能考虑

1. **数据采集频率**: 默认1秒更新一次，可配置
2. **历史数据限制**: 最多保存最近1000个数据点
3. **内存优化**: 使用循环缓冲区存储历史数据
4. **渲染优化**: 只在数据变化时重绘相关组件

## 配置系统

```rust
#[derive(Serialize, Deserialize)]
pub struct AppConfig {
    pub update_interval_ms: u64,
    pub max_history_points: usize,
    pub show_cpu_cores: bool,
    pub show_process_list: bool,
    pub window_size: (f32, f32),
}
```

## 错误处理策略

```rust
#[derive(Debug)]
pub enum SystemMonitorError {
    CollectorError(String),
    ConfigError(String),
    UiError(String),
}

pub type Result<T> = std::result::Result<T, SystemMonitorError>;