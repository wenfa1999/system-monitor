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
        };
        
        // 初始化系统信息管理器
        app.initialize_system_manager()?;
        
        log::info!("系统监控应用程序初始化完成");
        Ok(app)
    }
    
    /// 初始化系统信息管理器
    fn initialize_system_manager(&mut self) -> Result<()> {
        let config = self.config_manager.get();
        
        match SystemInfoManager::new(
            config.monitoring.refresh_interval_ms,
            config.monitoring.cpu_history_points,
        ) {
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
    
    /// 更新系统信息
    fn update_system_info(&mut self) -> Result<()> {
        if let Some(ref system_manager) = self.system_manager {
            // 获取系统快照
            let cpu_info = system_manager.get_cpu_info()?;
            let memory_info = system_manager.get_memory_info()?;
            let disk_info = system_manager.get_disk_info()?;
            let system_info = system_manager.get_system_info()?;
            
            // 创建系统快照
            let snapshot = SystemSnapshot::new(
                cpu_info,
                memory_info,
                disk_info,
                system_info,
                None, // 网络信息暂时为空
            );
            
            // 发送更新消息
            if let Some(ref sender) = self.message_sender {
                let _ = sender.send(AppMessage::SystemUpdate(snapshot));
            }
        }
        
        Ok(())
    }
    
    /// 处理定时更新
    fn handle_periodic_update(&mut self) {
        let config = self.config_manager.get();
        let update_interval = Duration::from_millis(config.monitoring.refresh_interval_ms);
        
        if self.last_update.elapsed() >= update_interval {
            if let Err(e) = self.update_system_info() {
                log::error!("更新系统信息失败: {}", e);
                if let Some(ref sender) = self.message_sender {
                    let _ = sender.send(AppMessage::Error(format!("系统信息更新失败: {}", e)));
                }
            }
            self.last_update = Instant::now();
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
    
    /// 渲染设置窗口
    fn render_settings_window(&mut self, ctx: &egui::Context) {
        if !self.app_state.show_settings {
            return;
        }
        
        egui::Window::new("设置")
            .default_width(400.0)
            .default_height(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("应用程序设置");
                ui.separator();
                
                // 监控设置
                ui.collapsing("监控设置", |ui| {
                    let mut config = self.config_manager.get().clone();
                    let mut changed = false;
                    
                    ui.horizontal(|ui| {
                        ui.label("刷新间隔 (毫秒):");
                        if ui.add(egui::Slider::new(&mut config.monitoring.refresh_interval_ms, 100..=5000)).changed() {
                            changed = true;
                        }
                    });
                    
                    if ui.checkbox(&mut config.monitoring.enable_cpu_monitoring, "启用CPU监控").changed() {
                        changed = true;
                    }
                    
                    if ui.checkbox(&mut config.monitoring.enable_memory_monitoring, "启用内存监控").changed() {
                        changed = true;
                    }
                    
                    if ui.checkbox(&mut config.monitoring.enable_disk_monitoring, "启用磁盘监控").changed() {
                        changed = true;
                    }
                    
                    if changed {
                        if let Err(e) = self.config_manager.update(|cfg| *cfg = config) {
                            log::error!("更新配置失败: {}", e);
                        } else if let Some(ref sender) = self.message_sender {
                            let _ = sender.send(AppMessage::ConfigUpdate);
                        }
                    }
                });
                
                // UI设置
                ui.collapsing("界面设置", |ui| {
                    let mut config = self.config_manager.get().clone();
                    let mut changed = false;
                    
                    ui.horizontal(|ui| {
                        ui.label("字体大小:");
                        if ui.add(egui::Slider::new(&mut config.ui.font_size, 8.0..=24.0)).changed() {
                            changed = true;
                        }
                    });
                    
                    if ui.checkbox(&mut config.ui.show_grid, "显示网格").changed() {
                        changed = true;
                    }
                    
                    if changed {
                        if let Err(e) = self.config_manager.update(|cfg| *cfg = config) {
                            log::error!("更新配置失败: {}", e);
                        }
                    }
                });
                
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("关闭").clicked() {
                        self.app_state.show_settings = false;
                    }
                    
                    if ui.button("重置为默认").clicked() {
                        if let Err(e) = self.config_manager.update(|cfg| cfg.reset_to_default()) {
                            log::error!("重置配置失败: {}", e);
                        } else if let Some(ref sender) = self.message_sender {
                            let _ = sender.send(AppMessage::ConfigUpdate);
                        }
                    }
                });
            });
    }
    
    /// 渲染关于窗口
    fn render_about_window(&mut self, ctx: &egui::Context) {
        if !self.app_state.show_about {
            return;
        }
        
        egui::Window::new("关于")
            .default_width(350.0)
            .default_height(250.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("系统监控工具");
                    ui.label("版本 0.1.0");
                    ui.separator();
                    
                    ui.label("基于Rust和egui构建的实时系统监控工具");
                    ui.label("提供CPU、内存、磁盘等系统信息的实时监控");
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        ui.label("运行时间:");
                        ui.label(format!("{:.1}秒", self.app_state.start_time.elapsed().as_secs_f32()));
                    });
                    
                    if let Some(ref snapshot) = self.app_state.current_snapshot {
                        ui.horizontal(|ui| {
                            ui.label("系统状态:");
                            ui.colored_label(
                                egui::Color32::from_rgb(
                                    (snapshot.get_health_status().color()[0] * 255.0) as u8,
                                    (snapshot.get_health_status().color()[1] * 255.0) as u8,
                                    (snapshot.get_health_status().color()[2] * 255.0) as u8,
                                ),
                                snapshot.get_health_status().description()
                            );
                        });
                    }
                    
                    ui.separator();
                    
                    if ui.button("关闭").clicked() {
                        self.app_state.show_about = false;
                    }
                });
            });
    }
}

impl eframe::App for SystemMonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理定时更新
        self.handle_periodic_update();
        
        // 处理消息队列
        self.process_messages();
        
        // 渲染主界面
        self.ui_manager.render(ctx, &self.app_state);
        
        // 渲染设置窗口
        self.render_settings_window(ctx);
        
        // 渲染关于窗口
        self.render_about_window(ctx);
    }
    
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        // 保存应用程序状态到存储
        if let Err(e) = self.config_manager.save() {
            log::error!("保存配置失败: {}", e);
        }
    }
}