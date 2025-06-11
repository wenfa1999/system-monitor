//! UI管理器
//! 
//! 负责协调和管理所有UI组件的渲染和状态。

use crate::config::AppConfig;
use crate::error::Result;
use crate::system::SystemSnapshot;
use crate::ui::{TabType, UiState, UiTheme, ColorScheme, MemoryTabRenderer, DiskTabRenderer, ProcessTabRenderer, NetworkTabRenderer};
use eframe::egui;
use std::sync::Arc;
use std::collections::HashMap;

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
    pub fn render(&mut self, ctx: &egui::Context, app_state: &crate::app::AppState) {
        // 渲染顶部菜单栏
        self.render_menu_bar(ctx);
        
        // 渲染侧边栏
        if self.state.show_sidebar {
            self.render_sidebar(ctx);
        }
        
        // 渲染主内容区域
        self.render_main_content(ctx);
        
        // 渲染状态栏
        self.render_status_bar(ctx, app_state);
    }
    
    /// 渲染菜单栏
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("设置").clicked() {
                        // 发送显示设置消息
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        // 发送退出消息
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
                        // 发送显示关于消息
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
    fn render_sidebar(&mut self, ctx: &egui::Context) {
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
                            self.state.active_tab = tab_type;
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
    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // 渲染标签页标题
            ui.horizontal(|ui| {
                ui.heading(self.state.active_tab.name());
                
                // 右对齐的刷新按钮
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🔄 刷新").clicked() {
                        // 发送刷新消息
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