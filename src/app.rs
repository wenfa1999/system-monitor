//! 主应用程序模块
//! 
//! 定义了系统监控工具的主应用程序结构和状态管理。

use crate::config::{AppConfig, ConfigManager};
use crate::error::{Result, SystemMonitorError, ErrorRecovery};
use crate::system::{SystemInfoManager, SystemSnapshot, SystemHealthStatus};
use crate::ui::{UiManager, TabType};
use eframe::egui;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// 主应用程序结构
pub struct SystemMonitorApp {
    /// 配置管理器
    config_manager: ConfigManager,
    /// 系统信息管理器
    system_manager: Option<SystemInfoManager>,
    /// UI管理器
    ui_manager: UiManager,
    /// 错误恢复处理器
    error_recovery: ErrorRecovery,
    /// 应用程序状态
    app_state: AppState,
    /// 最后更新时间
    last_update: Instant,
    /// 消息通道发送端
    message_sender: Option<mpsc::UnboundedSender<AppMessage>>,
    /// 消息通道接收端
    message_receiver: Option<mpsc::UnboundedReceiver<AppMessage>>,
    /// 用于取消后台任务的令牌
    cancellation_token: tokio_util::sync::CancellationToken,
}

/// 应用程序状态
#[derive(Debug, Clone)]
pub struct AppState {
    /// 是否正在运行
    pub is_running: bool,
    /// 当前系统快照
    pub current_snapshot: Option<SystemSnapshot>,
    /// 系统健康状态
    pub health_status: SystemHealthStatus,
    /// 错误信息
    pub last_error: Option<String>,
    /// 是否显示设置窗口
    pub show_settings: bool,
    /// 是否显示关于窗口
    pub show_about: bool,
    /// 当前活动标签页
    pub active_tab: TabType,
    /// 应用程序启动时间
    pub start_time: Instant,
}

/// 应用程序消息
#[derive(Debug, Clone)]
pub enum AppMessage {
    /// 系统信息更新
    SystemUpdate(SystemSnapshot),
    /// 配置更新
    ConfigUpdate,
    /// 错误发生
    Error(String),
    /// 切换标签页
    SwitchTab(TabType),
    /// 应用配置
    ApplyConfig(AppConfig),
    /// 显示设置
    ShowSettings,
    /// 隐藏设置
    HideSettings,
    /// 显示关于
    ShowAbout,
    /// 隐藏关于
    HideAbout,
    /// 退出应用
    Exit,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            is_running: true,
            current_snapshot: None,
            health_status: SystemHealthStatus::Good,
            last_error: None,
            show_settings: false,
            show_about: false,
            active_tab: TabType::Overview,
            start_time: Instant::now(),
        }
    }
}

impl SystemMonitorApp {
    /// 创建新的应用程序实例
    pub fn new(cc: &eframe::CreationContext<'_>, config: Arc<AppConfig>) -> Result<Self> {
        // 初始化配置管理器
        let config_manager = ConfigManager::new(true)?;
        
        // 创建消息通道
        let (message_sender, message_receiver) = mpsc::unbounded_channel();
        
        // 初始化UI管理器
        let ui_manager = UiManager::new(&cc.egui_ctx, config.clone())?;
        
        // 创建错误恢复处理器
        let error_recovery = ErrorRecovery::default();
        
        // 初始化应用程序状态
        let app_state = AppState::default();
        
        let mut app = Self {
            config_manager,
            system_manager: None,
            ui_manager,
            error_recovery,
            app_state,
            last_update: Instant::now(),
            message_sender: Some(message_sender),
            message_receiver: Some(message_receiver),
            cancellation_token: tokio_util::sync::CancellationToken::new(),
        };
        
        // 初始化系统信息管理器
        app.initialize_system_manager()?;
        
        // 启动后台数据采集任务
        app.start_background_collector();

        log::info!("系统监控应用程序初始化完成");
        Ok(app)
    }
    
    /// 初始化系统信息管理器
    fn initialize_system_manager(&mut self) -> Result<()> {
        match SystemInfoManager::new() {
            Ok(manager) => {
                self.system_manager = Some(manager);
                log::info!("系统信息管理器初始化成功");
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("系统信息管理器初始化失败: {}", e);
                log::error!("{}", error_msg);
                self.app_state.last_error = Some(error_msg.clone());
                Err(SystemMonitorError::Runtime(error_msg))
            }
        }
    }
    
    /// 处理应用程序消息
    fn handle_message(&mut self, message: AppMessage) {
        match message {
            AppMessage::SystemUpdate(snapshot) => {
                self.app_state.current_snapshot = Some(snapshot.clone());
                self.app_state.health_status = snapshot.get_health_status();
                self.ui_manager.update_system_data(snapshot);
            }
            AppMessage::ConfigUpdate => {
                if let Err(e) = self.reload_configuration() {
                    log::error!("重新加载配置失败: {}", e);
                    self.app_state.last_error = Some(format!("配置更新失败: {}", e));
                }
            }
            AppMessage::Error(error) => {
                log::error!("应用程序错误: {}", error);
                self.app_state.last_error = Some(error);
            }
            AppMessage::SwitchTab(tab) => {
                self.app_state.active_tab = tab;
                self.ui_manager.set_active_tab(tab);
            }
            AppMessage::ApplyConfig(new_config) => {
                if let Err(e) = self.config_manager.update(|cfg| *cfg = new_config) {
                    log::error!("更新配置失败: {}", e);
                } else if let Some(ref sender) = self.message_sender {
                    let _ = sender.send(AppMessage::ConfigUpdate);
                }
            }
            AppMessage::ShowSettings => {
                self.app_state.show_settings = true;
            }
            AppMessage::HideSettings => {
                self.app_state.show_settings = false;
            }
            AppMessage::ShowAbout => {
                self.app_state.show_about = true;
            }
            AppMessage::HideAbout => {
                self.app_state.show_about = false;
            }
            AppMessage::Exit => {
                self.app_state.is_running = false;
            }
        }
    }
    
    /// 重新加载配置
    fn reload_configuration(&mut self) -> Result<()> {
        // 重新加载配置管理器
        self.config_manager = ConfigManager::new(true)?;
        
        // 重新初始化系统管理器（如果配置发生变化）
        self.initialize_system_manager()?;
        
        // 更新UI管理器配置
        let config = Arc::new(self.config_manager.get().clone());
        self.ui_manager.update_config(config)?;
        
        log::info!("配置重新加载完成");
        Ok(())
    }
    
    /// 启动后台数据采集任务
    fn start_background_collector(&mut self) {
        if let (Some(system_manager), Some(sender)) = (self.system_manager.as_ref(), self.message_sender.as_ref()) {
            let system_manager = system_manager.clone();
            let sender = sender.clone();
            let config = self.config_manager.get().clone();
            let token = self.cancellation_token.clone();

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(config.monitoring.refresh_interval_ms));
                loop {
                    tokio::select! {
                        _ = token.cancelled() => {
                            break;
                        }
                        _ = interval.tick() => {
                            match system_manager.get_snapshot().await {
                                Ok(snapshot) => {
                                    if sender.send(AppMessage::SystemUpdate(snapshot)).is_err() {
                                        break; // Channel closed
                                    }
                                },
                                Err(e) => {
                                    if sender.send(AppMessage::Error(format!("数据采集失败: {}", e))).is_err() {
                                        break; // Channel closed
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }
    }
    
    /// 处理待处理的消息
    fn process_messages(&mut self) {
        let mut messages = Vec::new();
        if let Some(ref mut receiver) = self.message_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }
        
        for message in messages {
            self.handle_message(message);
        }
    }
    
}

impl eframe::App for SystemMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 不再调用 self.handle_periodic_update();
        self.process_messages();
        
        // 将 AppState 和 message_sender 传递给 UiManager
        // UiManager 现在负责所有渲染
        if let Some(sender) = &self.message_sender {
            self.ui_manager.render(ctx, &mut self.app_state, sender);
        }
    }
    
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // 保存应用程序状态到存储
        if let Err(e) = self.config_manager.save() {
            log::error!("保存配置失败: {}", e);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.cancellation_token.cancel();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::system::info::{CpuInfo, MemoryInfo, DiskInfo, SystemInfo};
    use std::sync::Arc;

    // Helper function to create a default AppState for testing
    fn default_app_state() -> AppState {
        AppState::default()
    }

    // Helper function to create a dummy SystemMonitorApp for testing handle_message
    fn test_app() -> SystemMonitorApp {
        let (tx, rx) = mpsc::unbounded_channel();
        SystemMonitorApp {
            config_manager: ConfigManager::new(false).unwrap(),
            system_manager: None,
            ui_manager: UiManager::new(&egui::Context::default(), Arc::new(AppConfig::default())).unwrap(),
            error_recovery: ErrorRecovery::default(),
            app_state: default_app_state(),
            last_update: Instant::now(),
            message_sender: Some(tx),
            message_receiver: Some(rx),
            cancellation_token: tokio_util::sync::CancellationToken::new(),
        }
    }

    #[test]
    fn test_handle_message_system_update() {
        let mut app = test_app();
        let snapshot = SystemSnapshot::new(
            CpuInfo::default(),
            MemoryInfo::default(),
            vec![DiskInfo::default()],
            SystemInfo::default(),
            None,
        );
        let message = AppMessage::SystemUpdate(snapshot.clone());
        app.handle_message(message);

        assert!(app.app_state.current_snapshot.is_some());
        if let Some(snap) = app.app_state.current_snapshot {
            assert_eq!(snap.cpu, snapshot.cpu);
        }
    }

    #[test]
    fn test_handle_message_switch_tab() {
        let mut app = test_app();
        assert_eq!(app.app_state.active_tab, TabType::Overview);
        let message = AppMessage::SwitchTab(TabType::Process);
        app.handle_message(message);
        assert_eq!(app.app_state.active_tab, TabType::Process);
    }

    #[test]
    fn test_handle_message_show_hide_settings() {
        let mut app = test_app();
        assert!(!app.app_state.show_settings);

        // Show
        let message_show = AppMessage::ShowSettings;
        app.handle_message(message_show);
        assert!(app.app_state.show_settings);

        // Hide
        let message_hide = AppMessage::HideSettings;
        app.handle_message(message_hide);
        assert!(!app.app_state.show_settings);
    }

    #[test]
    fn test_handle_message_exit() {
        let mut app = test_app();
        assert!(app.app_state.is_running);
        let message = AppMessage::Exit;
        app.handle_message(message);
        assert!(!app.app_state.is_running);
    }
}