# 系统监控工具改进规范

本文档概述了对现有系统监控工具代码库的建议改进。这些改进旨在提高模块化、性能、可维护性和稳健性，与项目既定的架构目标保持一致。

## 1. 改进领域

基于对代码库的分析，确定了以下四个主要的改进领域：

1.  **字体管理**: 当前的字体加载机制是脆弱的，并且硬编码了文件路径。
2.  **UI/应用逻辑耦合**: 核心应用逻辑 (`app.rs`) 包含了本应属于UI层的渲染代码。
3.  **数据采集异步性**: 数据采集是同步执行的，可能会阻塞UI线程。
4.  **配置更新流程**: 配置更新的流程可以更加清晰和单向。

---

## 2. 伪代码改进规范

以下是针对每个改进领域的详细伪代码规范。

### 2.1. 重构字体管理

**目标**: 使字体加载可配置、健壮，并提供明确的错误反馈。

**文件**: `src/main.rs`, `src/config/mod.rs`

```pseudocode
// file: src/config/mod.rs

// 在 WindowConfig 或新的 UiConfig 结构中添加字体配置
struct UiConfig {
    font_path: Option<String>, // 可选的自定义字体路径
    // ... 其他UI配置
}

// 在 AppConfig 中
struct AppConfig {
    // ...
    ui: UiConfig,
}

impl AppConfig {
    function default() -> Self {
        // ...
        ui: UiConfig {
            font_path: Some("assets/fonts/NotoSansSC-Regular.ttf".to_string()), // 默认捆绑字体
            // ...
        }
    }
}

// file: src/main.rs

function setup_custom_fonts(ctx: &egui::Context, config: &AppConfig) -> Result<(), String> {
    let mut fonts = egui::FontDefinitions::default();
    let mut font_loaded = false;

    // 1. 尝试从配置中加载字体
    if let Some(path) = &config.ui.font_path {
        if let Ok(font_data) = std::fs::read(path) {
            log::info("从配置路径加载字体: {}", path);
            fonts.font_data.insert("custom_font", egui::FontData::from_owned(font_data));
            set_font_families(&mut fonts, "custom_font");
            font_loaded = true;
        } else {
            log::warn("无法从配置路径加载字体: {}", path);
        }
    }

    // 2. 如果失败，尝试后备系统字体 (作为最后的手段)
    if !font_loaded {
        let system_fonts = ["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/simsun.ttc"];
        for path in system_fonts {
            if let Ok(font_data) = std::fs::read(path) {
                log::info("加载后备系统字体: {}", path);
                fonts.font_data.insert("fallback_font", egui::FontData::from_owned(font_data));
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

    Ok(())
}

// 在 main 函数中
async function main() {
    // ...
    // 在创建 app 之前
    if let Err(e) = setup_custom_fonts(&cc.egui_ctx, &config) {
        // 在UI中显示一个致命错误，因为没有字体无法继续
        // 这可以通过一个临时的、只有错误消息的 eframe app 来实现
        // 或者在 SystemMonitorApp 中添加一个初始化错误状态
        error!("字体初始化失败: {}", e);
        // 优雅地退出或显示错误窗口
        return;
    }

    let app = SystemMonitorApp::new(cc, Arc::new(config));
    // ...
}
```

### 2.2. 解耦UI和应用逻辑

**目标**: 将所有 `egui` 渲染逻辑从 `SystemMonitorApp` 移至 `UiManager`。

**文件**: `src/app.rs`, `src/ui/manager.rs`

```pseudocode
// file: src/app.rs

// 从 SystemMonitorApp 中移除 render_settings_window 和 render_about_window 方法

// AppMessage 枚举需要调整
enum AppMessage {
    // ...
    // 不再需要 Show/Hide 消息，因为状态在 AppState 中
    // 添加一个用于应用配置更改的消息
    ApplyConfig(AppConfig),
}

impl SystemMonitorApp {
    // ...
    function handle_message(&mut self, message: AppMessage) {
        match message {
            // ...
            AppMessage::ApplyConfig(new_config) => {
                if let Err(e) = self.config_manager.update(|cfg| *cfg = new_config) {
                    log::error!("更新配置失败: {}", e);
                } else {
                    // 触发重新加载
                    let _ = self.message_sender.send(AppMessage::ConfigUpdate);
                }
            }
        }
    }
}

impl eframe::App for SystemMonitorApp {
    function update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_periodic_update();
        self.process_messages();

        // 将 AppState 和 message_sender 传递给 UiManager
        // UiManager 现在负责所有渲染
        self.ui_manager.render(ctx, &mut self.app_state, &self.message_sender);
    }
}


// file: src/ui/manager.rs

struct UiManager {
    // ...
}

impl UiManager {
    // render 方法签名改变
    function render(&mut self, ctx: &egui::Context, app_state: &mut AppState, sender: &mpsc::UnboundedSender<AppMessage>) {
        // 渲染主面板
        self.render_main_panel(ctx, app_state, sender);

        // 根据状态渲染设置窗口
        if app_state.show_settings {
            self.render_settings_window(ctx, app_state, sender);
        }

        // 根据状态渲染关于窗口
        if app_state.show_about {
            self.render_about_window(ctx, app_state);
        }
    }

    // 实现 render_settings_window
    function render_settings_window(&mut self, ctx: &egui::Context, app_state: &mut AppState, sender: &mpsc::UnboundedSender<AppMessage>) {
        let mut open = app_state.show_settings;
        egui::Window::new("设置").open(&mut open).show(ctx, |ui| {
            // ... UI 逻辑 ...
            let mut config = self.config.as_ref().clone();
            let mut changed = false;

            // ... 当用户更改设置时 ...
            // if ui.add(...).changed() { changed = true; }

            if changed {
                // 发送消息而不是直接调用 config_manager
                let _ = sender.send(AppMessage::ApplyConfig(config));
            }

            if ui.button("关闭").clicked() {
                app_state.show_settings = false;
            }
        });
        if !open {
            app_state.show_settings = false;
        }
    }

    // 实现 render_about_window
    function render_about_window(&mut self, ctx: &egui::Context, app_state: &mut AppState) {
        // ... 类似地，使用 app_state.show_about 控制窗口 ...
    }
}
```

### 2.3. 实现异步数据采集

**目标**: 在后台线程中执行数据采集，避免阻塞UI。

**文件**: `src/app.rs`, `src/system/manager.rs`

```pseudocode
// file: src/system/manager.rs

// 将数据获取方法改为 async
impl SystemInfoManager {
    async function get_snapshot(&self) -> Result<SystemSnapshot> {
        // tokio::try_join! 可以并行执行异步任务
        let (cpu_info, memory_info, disk_info, system_info) = tokio::try_join!(
            self.get_cpu_info_async(),
            self.get_memory_info_async(),
            self.get_disk_info_async(),
            self.get_system_info_async()
        )?;

        Ok(SystemSnapshot::new(cpu_info, memory_info, disk_info, system_info, None))
    }

    // 将每个 get_* 方法重构为异步版本，例如:
    async function get_cpu_info_async(&self) -> Result<CpuInfo> {
        // 在 spawn_blocking 中运行同步的 sysinfo 调用
        tokio::task::spawn_blocking(move || {
            // ... 原来的同步代码 ...
        }).await?
    }
    // ... 其他 get_*_async 方法 ...
}


// file: src/app.rs

impl SystemMonitorApp {
    function new(...) -> Result<Self> {
        // ...
        let mut app = Self { ... };

        // 在 new 函数中启动后台数据采集任务
        app.start_background_collector();

        Ok(app)
    }

    function start_background_collector(&mut self) {
        if let (Some(system_manager), Some(sender)) = (self.system_manager.as_ref(), self.message_sender.as_ref()) {
            let system_manager = system_manager.clone(); // 需要 SystemInfoManager 实现 Clone
            let sender = sender.clone();
            let config = self.config_manager.get().clone();

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(config.monitoring.refresh_interval_ms));
                loop {
                    interval.tick().await;
                    match system_manager.get_snapshot().await {
                        Ok(snapshot) => {
                            let _ = sender.send(AppMessage::SystemUpdate(snapshot));
                        },
                        Err(e) => {
                            let _ = sender.send(AppMessage::Error(format!("数据采集失败: {}", e)));
                        }
                    }
                }
            });
        }
    }

    // 移除 handle_periodic_update 和 update_system_info 方法
    // 因为这个逻辑现在由后台任务处理
}

// eframe::App::update 方法不再需要调用 handle_periodic_update
impl eframe::App for SystemMonitorApp {
    function update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 不再调用 self.handle_periodic_update();
        self.process_messages();
        self.ui_manager.render(ctx, &mut self.app_state, &self.message_sender);
    }
}
```

---

## 3. 内存银行更新

在应用这些更改后，建议更新以下内存银行文件：

*   **`decisionLog.md`**: 添加一个新的决策条目，记录进行这些重构的原因，例如“为了提高性能和模块化，将数据采集重构为异步，并解耦UI和应用逻辑”。
*   **`systemPatterns.md`**: 更新或强调“异步数据采集”和“模型-视图-控制器（MVC）”模式的实现，以反映这些更改如何更好地遵循了这些模式。
*   **`activeContext.md`**: 将当前焦点更新为“代码重构和性能优化”，并记录已完成的改进。