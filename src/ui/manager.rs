//! UI管理器
//! 
//! 负责协调和管理所有UI组件的渲染和状态。

use crate::config::AppConfig;
use crate::error::Result;
use crate::system::SystemSnapshot;
use crate::app::{AppMessage, AppState};
use crate::ui::{TabType, UiState, UiTheme, ColorScheme, MemoryTabRenderer, DiskTabRenderer, ProcessTabRenderer, NetworkTabRenderer};
use eframe::egui;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// UI管理器
pub struct UiManager {
    /// UI状态
    state: UiState,
    /// 应用配置
    config: Arc<AppConfig>,
    /// 系统数据
    system_data: Option<SystemSnapshot>,
    /// 标签页渲染器
    tab_renderers: HashMap<TabType, Box<dyn TabRenderer>>,
}

/// 标签页渲染器特征
pub trait TabRenderer: Send + Sync {
    /// 渲染标签页内容
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>);
    
    /// 获取标签页标题
    fn title(&self) -> &str;
    
    /// 是否启用该标签页
    fn is_enabled(&self) -> bool {
        true
    }
}

impl UiManager {
    /// 创建新的UI管理器
    pub fn new(ctx: &egui::Context, config: Arc<AppConfig>) -> Result<Self> {
        // 根据配置设置主题
        let theme = match config.ui.theme {
            crate::config::Theme::Light => UiTheme::Light,
            crate::config::Theme::Dark => UiTheme::Dark,
            crate::config::Theme::Auto => {
                // 简单的自动主题检测，实际可以根据系统主题
                UiTheme::Dark
            }
        };
        
        theme.apply_to_context(ctx);
        
        let mut state = UiState::default();
        state.theme = theme;
        state.color_scheme = match theme {
            UiTheme::Light => ColorScheme::light(),
            UiTheme::Dark => ColorScheme::dark(),
        };
        state.font_size = config.ui.font_size;
        
        // 初始化标签页渲染器
        let mut tab_renderers: HashMap<TabType, Box<dyn TabRenderer>> = HashMap::new();
        tab_renderers.insert(TabType::Overview, Box::new(OverviewTabRenderer::new()));
        tab_renderers.insert(TabType::Cpu, Box::new(CpuTabRenderer::new()));
        tab_renderers.insert(TabType::Memory, Box::new(MemoryTabRenderer::new()));
        tab_renderers.insert(TabType::Disk, Box::new(DiskTabRenderer::new()));
        tab_renderers.insert(TabType::Process, Box::new(ProcessTabRenderer::new()));
        tab_renderers.insert(TabType::Network, Box::new(NetworkTabRenderer::new()));
        
        Ok(Self {
            state,
            config,
            system_data: None,
            tab_renderers,
        })
    }
    
    /// 更新配置
    pub fn update_config(&mut self, config: Arc<AppConfig>) -> Result<()> {
        self.config = config;
        self.state.font_size = self.config.ui.font_size;
        Ok(())
    }
    
    /// 更新系统数据
    pub fn update_system_data(&mut self, data: SystemSnapshot) {
        self.system_data = Some(data);
    }
    
    /// 设置活动标签页
    pub fn set_active_tab(&mut self, tab: TabType) {
        self.state.active_tab = tab;
    }
    
    /// 渲染主界面
    pub fn render(&mut self, ctx: &egui::Context, app_state: &mut AppState, sender: &mpsc::UnboundedSender<AppMessage>) {
        // 渲染顶部菜单栏
        self.render_menu_bar(ctx, app_state, sender);
        
        // 渲染侧边栏
        if self.state.show_sidebar {
            self.render_sidebar(ctx, sender);
        }
        
        // 渲染主内容区域
        self.render_main_content(ctx, sender);
        
        // 渲染状态栏
        self.render_status_bar(ctx, app_state);

        // 根据状态渲染设置窗口
        if app_state.show_settings {
            self.render_settings_window(ctx, app_state, sender);
        }

        // 根据状态渲染关于窗口
        if app_state.show_about {
            self.render_about_window(ctx, app_state);
        }
    }
    
    /// 渲染菜单栏
    fn render_menu_bar(&mut self, ctx: &egui::Context, _app_state: &mut AppState, sender: &mpsc::UnboundedSender<AppMessage>) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("设置").clicked() {
                        let _ = sender.send(AppMessage::ShowSettings);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        let _ = sender.send(AppMessage::Exit);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("视图", |ui| {
                    if ui.checkbox(&mut self.state.show_sidebar, "显示侧边栏").clicked() {
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    ui.menu_button("主题", |ui| {
                        if ui.selectable_label(self.state.theme == UiTheme::Light, "亮色主题").clicked() {
                            self.state.theme = UiTheme::Light;
                            self.state.color_scheme = ColorScheme::light();
                            self.state.theme.apply_to_context(ctx);
                            ui.close_menu();
                        }
                        if ui.selectable_label(self.state.theme == UiTheme::Dark, "暗色主题").clicked() {
                            self.state.theme = UiTheme::Dark;
                            self.state.color_scheme = ColorScheme::dark();
                            self.state.theme.apply_to_context(ctx);
                            ui.close_menu();
                        }
                    });
                });
                
                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        let _ = sender.send(AppMessage::ShowAbout);
                        ui.close_menu();
                    }
                });
                
                // 右对齐的系统状态
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref data) = self.system_data {
                        let health_color = data.get_health_status().color();
                        ui.colored_label(
                            egui::Color32::from_rgb(
                                (health_color[0] * 255.0) as u8,
                                (health_color[1] * 255.0) as u8,
                                (health_color[2] * 255.0) as u8,
                            ),
                            format!("● {}", data.get_health_status().description())
                        );
                    }
                });
            });
        });
    }
    
    /// 渲染侧边栏
    fn render_sidebar(&mut self, ctx: &egui::Context, sender: &mpsc::UnboundedSender<AppMessage>) {
        egui::SidePanel::left("sidebar")
            .default_width(self.state.sidebar_width)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                ui.heading("系统监控");
                ui.separator();
                
                // 渲染标签页导航
                for tab_type in TabType::all() {
                    let is_active = self.state.active_tab == tab_type;
                    let is_enabled = self.tab_renderers.get(&tab_type)
                        .map(|renderer| renderer.is_enabled())
                        .unwrap_or(true);
                    
                    ui.add_enabled_ui(is_enabled, |ui| {
                        if ui.selectable_label(is_active, tab_type.name()).clicked() {
                            let _ = sender.send(AppMessage::SwitchTab(tab_type));
                        }
                    });
                }
                
                ui.separator();
                
                // 系统信息摘要
                if let Some(ref data) = self.system_data {
                    ui.heading("系统摘要");
                    
                    ui.label(format!("CPU: {:.1}%", data.cpu.global_usage));
                    ui.label(format!("内存: {:.1}%", data.memory.usage_percent));
                    
                    if let Some(disk) = data.disks.first() {
                        ui.label(format!("磁盘: {:.1}%", disk.usage_percent));
                    }
                    
                    ui.separator();
                    ui.small(format!("更新时间: {}", 
                        data.timestamp.format("%H:%M:%S")));
                }
            });
    }
    
    /// 渲染主内容区域
    fn render_main_content(&mut self, ctx: &egui::Context, _sender: &mpsc::UnboundedSender<AppMessage>) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 渲染标签页标题
            ui.horizontal(|ui| {
                ui.heading(self.state.active_tab.name());
                
                // 右对齐的刷新按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 刷新").clicked() {
                        // 发送刷新消息 - 这将在异步重构中更有用
                    }
                });
            });
            
            ui.separator();
            
            // 渲染活动标签页内容
            if let Some(renderer) = self.tab_renderers.get_mut(&self.state.active_tab) {
                renderer.render(ui, self.system_data.as_ref());
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("标签页内容加载中...");
                });
            }
        });
    }
    
    /// 渲染状态栏
    fn render_status_bar(&mut self, ctx: &egui::Context, app_state: &crate::app::AppState) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // 左侧状态信息
                ui.label(format!("运行时间: {:.0}秒", app_state.start_time.elapsed().as_secs()));
                
                ui.separator();
                
                if let Some(ref error) = app_state.last_error {
                    ui.colored_label(self.state.color_scheme.error, format!("错误: {}", error));
                } else {
                    ui.colored_label(self.state.color_scheme.success, "运行正常");
                }
                
                // 右侧系统信息
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref data) = self.system_data {
                        ui.label(format!("{}核心", data.cpu.core_count));
                        ui.separator();
                        ui.label(crate::ui::UiUtils::format_bytes(data.memory.total));
                        ui.separator();
                        ui.label(&data.system.hostname);
                    }
                });
            });
        });
    }
    /// 渲染设置窗口
    fn render_settings_window(&mut self, ctx: &egui::Context, app_state: &mut AppState, sender: &mpsc::UnboundedSender<AppMessage>) {
        let mut open = app_state.show_settings;
        egui::Window::new("设置")
            .open(&mut open)
            .default_width(400.0)
            .default_height(300.0)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("应用程序设置");
                ui.separator();
                
                let mut config = self.config.as_ref().clone();
                let mut changed = false;

                // 监控设置
                ui.collapsing("监控设置", |ui| {
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
                });
                
                // UI设置
                ui.collapsing("界面设置", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("字体大小:");
                        if ui.add(egui::Slider::new(&mut config.ui.font_size, 8.0..=24.0)).changed() {
                            changed = true;
                        }
                    });
                    
                    if ui.checkbox(&mut config.ui.show_grid, "显示网格").changed() {
                        changed = true;
                    }
                });

                if changed {
                    // 发送消息而不是直接调用 config_manager
                    let _ = sender.send(AppMessage::ApplyConfig(config));
                }
                
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("关闭").clicked() {
                        app_state.show_settings = false;
                    }
                    
                    if ui.button("重置为默认").clicked() {
                        let _ = sender.send(AppMessage::ApplyConfig(AppConfig::default()));
                    }
                });
            });
        
        if !open {
            app_state.show_settings = false;
        }
    }
    
    /// 渲染关于窗口
    fn render_about_window(&mut self, ctx: &egui::Context, app_state: &mut AppState) {
        let mut open = app_state.show_about;
        egui::Window::new("关于")
            .open(&mut open)
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
                        ui.label(format!("{:.1}秒", app_state.start_time.elapsed().as_secs_f32()));
                    });
                    
                    if let Some(ref snapshot) = self.system_data {
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
                        app_state.show_about = false;
                    }
                });
            });
        if !open {
            app_state.show_about = false;
        }
    }
}

// 标签页渲染器实现

/// 概览标签页渲染器
pub struct OverviewTabRenderer;

impl OverviewTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for OverviewTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        if let Some(data) = system_data {
            ui.columns(2, |columns| {
                // 左列 - CPU和内存
                columns[0].heading("性能概览");
                columns[0].separator();
                
                crate::ui::UiUtils::progress_bar(
                    &mut columns[0], 
                    data.cpu.global_usage, 
                    100.0, 
                    "CPU使用率"
                );
                
                crate::ui::UiUtils::progress_bar(
                    &mut columns[0], 
                    data.memory.usage_percent as f32, 
                    100.0, 
                    "内存使用率"
                );
                
                // 右列 - 系统信息
                columns[1].heading("系统信息");
                columns[1].separator();
                
                crate::ui::UiUtils::metric_display(
                    &mut columns[1], 
                    "操作系统", 
                    &format!("{} {}", data.system.os_name, data.system.os_version),
                    None
                );
                
                crate::ui::UiUtils::metric_display(
                    &mut columns[1], 
                    "主机名", 
                    &data.system.hostname,
                    None
                );
                
                crate::ui::UiUtils::metric_display(
                    &mut columns[1], 
                    "运行时间", 
                    &crate::ui::UiUtils::format_duration(data.system.uptime),
                    None
                );
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("正在加载系统数据...");
            });
        }
    }
    
    fn title(&self) -> &str {
        "概览"
    }
}

/// CPU标签页渲染器
pub struct CpuTabRenderer;

impl CpuTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for CpuTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        if let Some(data) = system_data {
            ui.heading(format!("CPU信息 - {}核心", data.cpu.core_count));
            ui.separator();
            
            // 全局CPU使用率
            crate::ui::UiUtils::progress_bar(
                ui, 
                data.cpu.global_usage, 
                100.0, 
                "总体CPU使用率"
            );
            
            ui.separator();
            
            // CPU核心详情
            ui.heading("CPU核心详情");
            for (i, core) in data.cpu.cores.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("核心 {}: ", i));
                    ui.add(egui::ProgressBar::new(core.usage / 100.0)
                        .fill(crate::ui::UiUtils::get_usage_color(core.usage as f64)));
                    ui.label(format!("{:.1}%", core.usage));
                    ui.label(format!("@ {}", crate::ui::UiUtils::format_frequency(core.frequency * 1_000_000)));
                });
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("正在加载CPU数据...");
            });
        }
    }
    
    fn title(&self) -> &str {
        "CPU"
    }
}