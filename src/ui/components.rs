//! UI组件模块
//! 
//! 提供可重用的UI组件。

use crate::ui::{UiUtils, TabRenderer};
use crate::system::SystemSnapshot;
use eframe::egui;

/// 内存标签页渲染器
pub struct MemoryTabRenderer;

impl MemoryTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for MemoryTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        if let Some(data) = system_data {
            ui.heading("内存信息");
            ui.separator();
            
            // 内存使用概览
            UiUtils::progress_bar(
                ui, 
                data.memory.usage_percent as f32, 
                100.0, 
                "内存使用率"
            );
            
            ui.separator();
            
            // 内存详细信息
            ui.columns(2, |columns| {
                columns[0].heading("内存统计");
                UiUtils::metric_display(
                    &mut columns[0], 
                    "总内存", 
                    &UiUtils::format_bytes(data.memory.total),
                    None
                );
                UiUtils::metric_display(
                    &mut columns[0], 
                    "已使用", 
                    &UiUtils::format_bytes(data.memory.used),
                    Some(UiUtils::get_usage_color(data.memory.usage_percent))
                );
                UiUtils::metric_display(
                    &mut columns[0], 
                    "可用", 
                    &UiUtils::format_bytes(data.memory.available),
                    None
                );
                UiUtils::metric_display(
                    &mut columns[0], 
                    "空闲", 
                    &UiUtils::format_bytes(data.memory.free),
                    None
                );
                
                columns[1].heading("使用率分析");
                columns[1].label(format!("使用率: {:.1}%", data.memory.usage_percent));
                
                let status = if data.memory.usage_percent < 50.0 {
                    ("正常", egui::Color32::GREEN)
                } else if data.memory.usage_percent < 80.0 {
                    ("注意", egui::Color32::YELLOW)
                } else {
                    ("警告", egui::Color32::RED)
                };
                
                UiUtils::status_indicator(&mut columns[1], status.0, status.1);
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("正在加载内存数据...");
            });
        }
    }
    
    fn title(&self) -> &str {
        "内存"
    }
}

/// 磁盘标签页渲染器
pub struct DiskTabRenderer;

impl DiskTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for DiskTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        if let Some(data) = system_data {
            ui.heading("磁盘信息");
            ui.separator();
            
            if data.disks.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("未检测到磁盘");
                });
                return;
            }
            
            // 磁盘列表
            for disk in &data.disks {
                UiUtils::info_card(ui, &disk.name, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            UiUtils::metric_display(ui, "挂载点", &disk.mount_point, None);
                            UiUtils::metric_display(ui, "文件系统", &disk.file_system, None);
                            UiUtils::metric_display(ui, "总容量", &UiUtils::format_bytes(disk.total_space), None);
                            UiUtils::metric_display(ui, "已使用", &UiUtils::format_bytes(disk.used_space), None);
                            UiUtils::metric_display(ui, "可用空间", &UiUtils::format_bytes(disk.available_space), None);
                        });
                        
                        ui.separator();
                        
                        ui.vertical(|ui| {
                            UiUtils::progress_bar(
                                ui, 
                                disk.usage_percent as f32, 
                                100.0, 
                                "使用率"
                            );
                        });
                    });
                });
                
                ui.add_space(8.0);
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("正在加载磁盘数据...");
            });
        }
    }
    
    fn title(&self) -> &str {
        "磁盘"
    }
}

/// 进程标签页渲染器
pub struct ProcessTabRenderer;

impl ProcessTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for ProcessTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        ui.heading("进程信息");
        ui.separator();
        
        // 暂时显示占位内容
        ui.label("进程监控功能开发中...");
        ui.label("将显示:");
        ui.label("• 进程列表");
        ui.label("• CPU和内存使用情况");
        ui.label("• 进程状态");
        ui.label("• 进程管理操作");
    }
    
    fn title(&self) -> &str {
        "进程"
    }
}

/// 网络标签页渲染器
pub struct NetworkTabRenderer;

impl NetworkTabRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl TabRenderer for NetworkTabRenderer {
    fn render(&mut self, ui: &mut egui::Ui, system_data: Option<&SystemSnapshot>) {
        ui.heading("网络信息");
        ui.separator();
        
        if let Some(data) = system_data {
            if let Some(ref networks) = data.networks {
                if networks.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label("未检测到网络接口");
                    });
                    return;
                }
                
                // 网络接口列表
                for network in networks {
                    UiUtils::info_card(ui, &network.name, |ui| {
                        ui.columns(2, |columns| {
                            columns[0].heading("接收");
                            UiUtils::metric_display(
                                &mut columns[0], 
                                "字节数", 
                                &UiUtils::format_bytes(network.bytes_received),
                                None
                            );
                            UiUtils::metric_display(
                                &mut columns[0], 
                                "包数", 
                                &network.packets_received.to_string(),
                                None
                            );
                            UiUtils::metric_display(
                                &mut columns[0], 
                                "错误数", 
                                &network.errors_received.to_string(),
                                if network.errors_received > 0 { Some(egui::Color32::RED) } else { None }
                            );
                            
                            columns[1].heading("发送");
                            UiUtils::metric_display(
                                &mut columns[1], 
                                "字节数", 
                                &UiUtils::format_bytes(network.bytes_sent),
                                None
                            );
                            UiUtils::metric_display(
                                &mut columns[1], 
                                "包数", 
                                &network.packets_sent.to_string(),
                                None
                            );
                            UiUtils::metric_display(
                                &mut columns[1], 
                                "错误数", 
                                &network.errors_sent.to_string(),
                                if network.errors_sent > 0 { Some(egui::Color32::RED) } else { None }
                            );
                        });
                    });
                    
                    ui.add_space(8.0);
                }
            } else {
                ui.label("网络信息不可用");
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("正在加载网络数据...");
            });
        }
    }
    
    fn title(&self) -> &str {
        "网络"
    }
}