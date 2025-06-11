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
            // 在创建 app 之前设置字体
            if let Err(e) = setup_custom_fonts(&cc.egui_ctx, &config) {
                // 字体初始化失败是一个致命错误
                error!("字体初始化失败: {}", e);
                // 在UI中显示一个致命错误，因为没有字体无法继续
                // 这里可以创建一个临时的错误显示应用
                // 为简单起见，我们直接恐慌
                panic!("字体初始化失败: {}", e);
            }

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
fn setup_custom_fonts(ctx: &egui::Context, config: &AppConfig) -> Result<(), String> {
    let mut fonts = egui::FontDefinitions::default();
    let mut font_loaded = false;

    // 1. 尝试从配置中加载字体
    if let Some(path) = &config.ui.font_path {
        if let Ok(font_data) = std::fs::read(path) {
            log::info!("从配置路径加载字体: {}", path);
            fonts.font_data.insert("custom_font".to_owned(), Arc::new(egui::FontData::from_owned(font_data)));
            set_font_families(&mut fonts, "custom_font");
            font_loaded = true;
        } else {
            log::warn!("无法从配置路径加载字体: {}", path);
        }
    }

    // 2. 如果失败，尝试后备系统字体
    if !font_loaded {
        let system_fonts = ["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/simsun.ttc"];
        for path in system_fonts {
            if let Ok(font_data) = std::fs::read(path) {
                log::info!("加载后备系统字体: {}", path);
                fonts.font_data.insert("fallback_font".to_owned(), Arc::new(egui::FontData::from_owned(font_data)));
                set_font_families(&mut fonts, "fallback_font");
                font_loaded = true;
                break;
            }
        }
    }

    ctx.set_fonts(fonts);

    if !font_loaded {
        return Err("无法加载任何有效的中文字体。请检查配置文件中的 `font_path` 或确保系统字体可用。".to_string());
    }

    log::info!("字体设置完成");
    Ok(())
}

/// 辅助函数，用于设置字体族
fn set_font_families(fonts: &mut egui::FontDefinitions, font_name: &str) {
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, font_name.to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(font_name.to_owned());
}