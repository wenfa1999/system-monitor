//! UI模块
//! 
//! 负责用户界面的渲染和交互处理。

pub mod manager;
pub mod components;
pub mod charts;
pub mod tabs;

pub use manager::*;
pub use components::*;

use eframe::egui;

/// 标签页类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabType {
    Overview,
    Cpu,
    Memory,
    Disk,
    Process,
    Network,
}

impl TabType {
    /// 获取标签页名称
    pub fn name(&self) -> &'static str {
        match self {
            TabType::Overview => "概览",
            TabType::Cpu => "CPU",
            TabType::Memory => "内存",
            TabType::Disk => "磁盘",
            TabType::Process => "进程",
            TabType::Network => "网络",
        }
    }

    /// 获取所有标签页
    pub fn all() -> Vec<TabType> {
        vec![
            TabType::Overview,
            TabType::Cpu,
            TabType::Memory,
            TabType::Disk,
            TabType::Process,
            TabType::Network,
        ]
    }
}

/// UI主题
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTheme {
    Light,
    Dark,
}

impl UiTheme {
    /// 应用主题到egui上下文
    pub fn apply_to_context(&self, ctx: &egui::Context) {
        match self {
            UiTheme::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            UiTheme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }
    }
}

/// UI颜色方案
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub primary: egui::Color32,
    pub secondary: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub error: egui::Color32,
    pub background: egui::Color32,
    pub surface: egui::Color32,
    pub text_primary: egui::Color32,
    pub text_secondary: egui::Color32,
}

impl ColorScheme {
    /// 创建亮色主题配色方案
    pub fn light() -> Self {
        Self {
            primary: egui::Color32::from_rgb(33, 150, 243),
            secondary: egui::Color32::from_rgb(156, 39, 176),
            success: egui::Color32::from_rgb(76, 175, 80),
            warning: egui::Color32::from_rgb(255, 193, 7),
            error: egui::Color32::from_rgb(244, 67, 54),
            background: egui::Color32::from_rgb(250, 250, 250),
            surface: egui::Color32::WHITE,
            text_primary: egui::Color32::from_rgb(33, 33, 33),
            text_secondary: egui::Color32::from_rgb(117, 117, 117),
        }
    }

    /// 创建暗色主题配色方案
    pub fn dark() -> Self {
        Self {
            primary: egui::Color32::from_rgb(33, 150, 243),
            secondary: egui::Color32::from_rgb(156, 39, 176),
            success: egui::Color32::from_rgb(76, 175, 80),
            warning: egui::Color32::from_rgb(255, 193, 7),
            error: egui::Color32::from_rgb(244, 67, 54),
            background: egui::Color32::from_rgb(18, 18, 18),
            surface: egui::Color32::from_rgb(33, 33, 33),
            text_primary: egui::Color32::WHITE,
            text_secondary: egui::Color32::from_rgb(158, 158, 158),
        }
    }
}

/// UI状态
#[derive(Debug, Clone)]
pub struct UiState {
    pub active_tab: TabType,
    pub show_sidebar: bool,
    pub sidebar_width: f32,
    pub font_size: f32,
    pub theme: UiTheme,
    pub color_scheme: ColorScheme,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            active_tab: TabType::Overview,
            show_sidebar: true,
            sidebar_width: 200.0,
            font_size: 14.0,
            theme: UiTheme::Dark,
            color_scheme: ColorScheme::dark(),
        }
    }
}

/// UI工具函数
pub struct UiUtils;

impl UiUtils {
    /// 格式化字节数为人类可读格式
    pub fn format_bytes(bytes: u64) -> String {
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

    /// 格式化百分比
    pub fn format_percentage(value: f64) -> String {
        format!("{:.1}%", value)
    }

    /// 格式化频率（Hz）
    pub fn format_frequency(hz: u64) -> String {
        if hz >= 1_000_000_000 {
            format!("{:.1} GHz", hz as f64 / 1_000_000_000.0)
        } else if hz >= 1_000_000 {
            format!("{:.1} MHz", hz as f64 / 1_000_000.0)
        } else if hz >= 1_000 {
            format!("{:.1} KHz", hz as f64 / 1_000.0)
        } else {
            format!("{} Hz", hz)
        }
    }

    /// 格式化时间间隔
    pub fn format_duration(seconds: u64) -> String {
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

    /// 获取使用率对应的颜色
    pub fn get_usage_color(usage_percent: f64) -> egui::Color32 {
        match usage_percent {
            x if x < 30.0 => egui::Color32::from_rgb(76, 175, 80),   // 绿色
            x if x < 60.0 => egui::Color32::from_rgb(255, 193, 7),   // 黄色
            x if x < 80.0 => egui::Color32::from_rgb(255, 152, 0),   // 橙色
            _ => egui::Color32::from_rgb(244, 67, 54),                // 红色
        }
    }

    /// 创建进度条
    pub fn progress_bar(ui: &mut egui::Ui, value: f32, max_value: f32, label: &str) -> egui::Response {
        let progress = (value / max_value).clamp(0.0, 1.0);
        let color = Self::get_usage_color((progress * 100.0) as f64);
        
        ui.horizontal(|ui| {
            ui.label(label);
            ui.add(egui::ProgressBar::new(progress).fill(color));
            ui.label(format!("{:.1}%", progress * 100.0));
        }).response
    }

    /// 创建状态指示器
    pub fn status_indicator(ui: &mut egui::Ui, status: &str, color: egui::Color32) -> egui::Response {
        ui.horizontal(|ui| {
            ui.add(egui::widgets::Button::new("●").fill(color).small());
            ui.label(status);
        }).response
    }

    /// 创建信息卡片
    pub fn info_card<R>(
        ui: &mut egui::Ui,
        title: &str,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<R> {
        egui::Frame::NONE
            .fill(ui.visuals().panel_fill)
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.heading(title);
                    ui.separator();
                    content(ui)
                }).inner
            })
    }

    /// 创建度量显示
    pub fn metric_display(ui: &mut egui::Ui, label: &str, value: &str, color: Option<egui::Color32>) {
        ui.horizontal(|ui| {
            ui.label(format!("{}:", label));
            if let Some(color) = color {
                ui.colored_label(color, value);
            } else {
                ui.label(value);
            }
        });
    }

    /// 创建表格行
    pub fn table_row(ui: &mut egui::Ui, columns: &[&str]) {
        ui.horizontal(|ui| {
            for (i, column) in columns.iter().enumerate() {
                if i > 0 {
                    ui.separator();
                }
                ui.label(*column);
            }
        });
    }

    /// 创建可折叠区域
    pub fn collapsing_section<R>(
        ui: &mut egui::Ui,
        title: &str,
        _default_open: bool,
        content: impl FnOnce(&mut egui::Ui) -> R,
    ) -> Option<R> {
        ui.collapsing(title, |ui| content(ui))
            .body_returned
    }
}

/// UI响应式布局助手
pub struct ResponsiveLayout;

impl ResponsiveLayout {
    /// 计算响应式列数
    pub fn calculate_columns(available_width: f32, min_column_width: f32) -> usize {
        ((available_width / min_column_width) as usize).max(1)
    }

    /// 创建响应式网格
    pub fn grid<R>(
        ui: &mut egui::Ui,
        items: &[impl Fn(&mut egui::Ui) -> R],
        min_column_width: f32,
    ) -> Vec<R> {
        let available_width = ui.available_width();
        let columns = Self::calculate_columns(available_width, min_column_width);
        let mut results = Vec::new();

        for chunk in items.chunks(columns) {
            ui.horizontal(|ui| {
                for item in chunk {
                    let result = ui.vertical(|ui| item(ui)).inner;
                    results.push(result);
                }
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(UiUtils::format_bytes(0), "0 B");
        assert_eq!(UiUtils::format_bytes(1024), "1.0 KB");
        assert_eq!(UiUtils::format_bytes(1048576), "1.0 MB");
        assert_eq!(UiUtils::format_bytes(1073741824), "1.0 GB");
    }

    #[test]
    fn test_format_percentage() {
        assert_eq!(UiUtils::format_percentage(25.5), "25.5%");
        assert_eq!(UiUtils::format_percentage(100.0), "100.0%");
    }

    #[test]
    fn test_tab_type_name() {
        assert_eq!(TabType::Overview.name(), "概览");
        assert_eq!(TabType::Cpu.name(), "CPU");
        assert_eq!(TabType::Memory.name(), "内存");
    }

    #[test]
    fn test_responsive_layout() {
        assert_eq!(ResponsiveLayout::calculate_columns(800.0, 200.0), 4);
        assert_eq!(ResponsiveLayout::calculate_columns(150.0, 200.0), 1);
    }
}