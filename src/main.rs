//! System Monitor - 基于Rust+egui的系统监控工具
//! 
//! 这是应用程序的主入口点，负责初始化应用程序并启动GUI。

use eframe::egui;
use log::{error, info};
use std::sync::Arc;

mod app;
mod config;
mod error;
mod system;
mod ui;
mod utils;

use app::SystemMonitorApp;
use config::AppConfig;
use error::SystemMonitorError;

/// 应用程序主函数
#[tokio::main]
async fn main() -> Result<(), SystemMonitorError> {
    // 初始化日志系统
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("启动系统监控工具...");

    // 加载应用程序配置
    let config = match AppConfig::load() {
        Ok(config) => {
            info!("成功加载配置文件");
            config
        }
        Err(e) => {
            error!("加载配置失败，使用默认配置: {}", e);
            AppConfig::default()
        }
    };

    // 设置eframe选项
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.window.width, config.window.height])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(load_icon())
            .with_resizable(true),
        ..Default::default()
    };

    // 启动应用程序
    let result = eframe::run_native(
        "系统监控工具",
        options,
        Box::new(|cc: &eframe::CreationContext| -> Result<Box<dyn eframe::App>, Box<dyn std::error::Error + Send + Sync>> {
            // 设置自定义字体（支持中文）
            setup_custom_fonts(&cc.egui_ctx);
            
            // 创建应用程序实例
            match SystemMonitorApp::new(cc, Arc::new(config)) {
                Ok(app) => Ok(Box::new(app)),
                Err(e) => Err(Box::new(e)),
            }
        }),
    );

    match result {
        Ok(_) => {
            info!("应用程序正常退出");
            Ok(())
        }
        Err(e) => {
            error!("应用程序运行错误: {}", e);
            Err(SystemMonitorError::Runtime(format!("eframe错误: {}", e)))
        }
    }
}

/// 加载应用程序图标
fn load_icon() -> egui::IconData {
    // 这里可以加载自定义图标，暂时返回默认图标
    egui::IconData {
        rgba: vec![255; 32 * 32 * 4], // 32x32 白色图标
        width: 32,
        height: 32,
    }
}

/// 设置自定义字体以支持中文显示
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试加载自定义字体，如果失败则使用系统字体
    let font_data = match std::fs::read("assets/fonts/NotoSansSC-Regular.ttf") {
        Ok(data) => {
            log::info!("成功加载自定义中文字体");
            egui::FontData::from_owned(data)
        }
        Err(_) => {
            log::warn!("未找到自定义字体文件，尝试使用系统字体");
            // 尝试使用Windows系统自带的微软雅黑字体
            match std::fs::read("C:/Windows/Fonts/msyh.ttc") {
                Ok(data) => {
                    log::info!("使用系统微软雅黑字体");
                    egui::FontData::from_owned(data)
                }
                Err(_) => {
                    // 如果系统字体也找不到，尝试其他常见中文字体
                    match std::fs::read("C:/Windows/Fonts/simsun.ttc") {
                        Ok(data) => {
                            log::info!("使用系统宋体字体");
                            egui::FontData::from_owned(data)
                        }
                        Err(_) => {
                            log::warn!("无法找到合适的中文字体，使用默认字体");
                            return; // 使用默认字体
                        }
                    }
                }
            }
        }
    };

    // 添加中文字体支持
    fonts.font_data.insert("chinese_font".to_owned(), Arc::new(font_data));

    // 设置字体优先级 - 将中文字体放在最前面
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "chinese_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("chinese_font".to_owned());

    ctx.set_fonts(fonts);
    log::info!("字体设置完成");
}