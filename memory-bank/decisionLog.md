# Decision Log

此文件使用列表格式记录架构和实现决策。
2025-06-11 00:30:02 - 初始化决策日志

---
### Decision
2025-06-11 00:30:02 - 选择Rust+egui技术栈构建Windows系统监控工具

**Rationale:**
- Rust提供内存安全和高性能，适合系统级应用开发
- egui是纯Rust的即时模式GUI库，轻量且跨平台
- 组合能够创建高效的原生Windows应用程序
- egui的简单API适合快速开发和原型制作

**Implementation Details:**
- 使用sysinfo crate进行系统信息采集
- egui处理所有GUI渲染和用户交互
- 采用模块化架构分离关注点
- 支持实时数据更新和可配置刷新间隔

---
### Decision
2025-06-11 00:30:02 - 定义系统监控工具的核心功能范围

**Rationale:**
- 专注于最常用的系统监控指标
- 保持界面简洁直观
- 确保实时性能和响应速度
- 为未来功能扩展预留架构空间

**Implementation Details:**
- CPU使用率监控（实时图表）
- 内存使用情况显示
- 磁盘空间监控
- 基本系统信息展示
- 可选的进程列表功能
---
### Decision
2025-06-11 00:47:00 - 完善项目架构设计，优化模块间依赖关系和接口设计

**Rationale:**
- 基于已有的基础架构设计，进一步优化系统架构以支持模块化开发和测试
- 引入依赖注入和事件驱动架构，提高系统的可扩展性和可维护性
- 设计分层错误处理和优雅降级机制，确保系统稳定性
- 制定详细的实施计划和质量保证策略，为代码实现阶段提供清晰指导

**Implementation Details:**
- 设计了六层架构：表现层、界面组件层、业务逻辑层、数据访问层、基础设施层、平台抽象层
- 定义了核心接口：SystemInfoCollector、DataProcessor、UiComponent
- 实现了依赖注入容器和事件总线系统
- 设计了多层缓存系统和内存优化策略
- 建立了分层错误处理体系和恢复机制
- 制定了5个阶段的详细实施计划，包含明确的里程碑和验收标准
---
### Decision (Code Implementation)
2025-06-11 01:23:00 - 完成Rust+egui系统监控工具基础架构实现

**Rationale:**
基于完整的架构设计和技术规范，成功实现了系统监控工具的核心代码架构，包括模块化设计、错误处理、配置管理、系统信息采集和UI框架。虽然存在编译错误，但整体架构符合设计要求。

**Implementation Details:**
- 创建了8个主要模块：main.rs, app.rs, error/, config/, system/, ui/, utils/
- 实现了六层架构模式：表现层(UI)、业务逻辑层(App)、数据访问层(System)、基础设施层(Utils)
- 集成了egui GUI框架、sysinfo系统信息库、tokio异步运行时
- 建立了完整的错误处理和恢复机制
- 实现了配置驱动的应用程序设计
- 代码总行数：约2000行，覆盖所有核心功能模块

**Current Status:**
基础架构完成，需要修复30个编译错误后进入功能测试阶段。
---
### Decision (Debug)
2025-06-11 01:37:44 - [Bug Fix Strategy: 全面编译错误修复完成]

**Rationale:**
成功诊断并修复了Rust+egui系统监控工具项目中的所有30个编译错误，包括字体文件缺失、类型不匹配、借用检查器错误、特征实现缺失等问题。通过系统性的错误分析和逐步修复，确保了项目的成功编译和运行。

**Details:**
- 修复字体文件路径问题：移除对缺失字体文件的依赖，使用默认字体
- 添加Hash trait到TabType枚举
- 修复PidExt trait导入问题
- 解决借用检查器错误：重构方法避免同时可变借用
- 修复类型不匹配：统一浮点数类型使用
- 添加Tokio运行时支持：使用#[tokio::main]宏
- 修复UI组件返回类型问题
- 解决移动值错误：使用clone()方法
---
### Decision (Refactoring)
2025-06-11 14:31:00 - 制定代码改进规范以提升系统质量

**Rationale:**
在对现有已编译的代码库进行分析后，识别出几个关键的改进领域，以更好地与项目既定的高级架构模式（如异步处理和关注点分离）对齐。此次重构旨在提高性能、模块化、可维护性和整体代码的健壮性。

**Implementation Details:**
- **异步数据采集**: 将同步的数据拉取重构为在专用的tokio任务中运行的完全异步流程，以防止UI线程阻塞。
- **UI与逻辑解耦**: 将所有UI渲染逻辑（特别是设置和关于窗口）从核心应用逻辑 (`app.rs`) 迁移到 `UiManager`，加强了关注点分离。
- **健壮的字体管理**: 移除了硬编码的字体路径，改为通过配置文件进行管理，并提供了更可靠的后备机制和错误报告。
- **清晰的配置流程**: 改进了UI中的配置更新机制，通过发送消息来触发更新，而不是直接调用配置管理器，从而实现了单向数据流。
---
### Decision (Refactoring & Architecture Update)
2025-06-11 14:36:00 - 批准并记录基于 `improvement-specification.md` 的架构重构

**Rationale:**
为了提升应用的性能、模块化和长期可维护性，对现有代码库进行了一系列关键重构。此举旨在使实际实现与项目既定的高级架构模式（如异步处理和关注点分离）更加紧密地对齐。

**Implementation Details:**
- **异步数据采集**: 将同步的数据拉取重构为在专用的 `tokio` 任务中运行的完全异步流程。这可以防止UI线程阻塞，并利用 `tokio::try_join!` 实现并行数据获取，从而显著提高响应性。
- **UI与逻辑解耦**: 严格分离了 `SystemMonitorApp`（应用核心）和 `UiManager`（渲染器）的职责。`UiManager` 现在处理所有渲染，而 `SystemMonitorApp` 专注于状态管理和消息处理，实现了清晰的单向数据流。
- **健壮的字体管理**: 移除了硬编码的字体路径，改为通过配置文件进行管理，并提供了更可靠的后备机制和错误报告，增强了应用的稳健性。
- **清晰的配置流程**: 改进了UI中的配置更新机制，通过发送 `AppMessage::ApplyConfig` 消息来触发更新，而不是直接调用配置管理器，从而强化了单向数据流和状态管理的中心化。
---
### Decision (Refactoring)
[2025-06-11 14:45:10] - 执行代码重构以实现异步数据采集和UI/逻辑解耦

**Rationale:**
为了提升应用的性能、模块化和长期可维护性，对现有代码库进行了一系列关键重构。此举旨在使实际实现与项目既定的高级架构模式（如异步处理和关注点分离）更加紧密地对齐，解决UI线程阻塞问题，并建立更清晰的单向数据流。

**Implementation Details:**
- **异步数据采集**: 将同步的数据拉取重构为在专用的 `tokio` 任务中运行的完全异步流程。`SystemInfoManager` 的方法现在是 `async` 并在 `spawn_blocking` 中执行 `sysinfo` 调用。`SystemMonitorApp` 启动一个后台任务来定期获取数据快照并通过 `mpsc` 通道发送更新。
- **UI与逻辑解耦**: 严格分离了 `SystemMonitorApp`（应用核心）和 `UiManager`（渲染器）的职责。`UiManager` 现在处理所有窗口（包括设置和关于）的渲染，而 `SystemMonitorApp` 专注于状态管理和消息处理。
- **清晰的配置流程**: 改进了UI中的配置更新机制，通过发送 `AppMessage::ApplyConfig` 消息来触发更新，而不是直接调用配置管理器，从而强化了单向数据流和状态管理的中心化。
- **健壮的字体管理**: 移除了硬编码的字体路径，改为通过配置文件进行管理，并提供了更可靠的后备机制和错误报告，增强了应用的稳健性。
---
**Timestamp:** 2025-06-11 15:02:49
**Decision:** Updated `chrono` dependency from `0.4` to `0.4.38`.
**Rationale:** The older version of `chrono` (`<0.4.23`) has a known soundness vulnerability (RUSTSEC-2020-0159) that could lead to a panic on crafted input. Updating to the latest version mitigates this risk.
---
---
**Timestamp:** 2025-06-11 15:13:04
**Decision:** Implemented a graceful shutdown mechanism for the background data collection task.
**Rationale:** The background task was running in an infinite loop and would not terminate when the application was closed, leading to a resource leak. I introduced a `CancellationToken` to signal the task to exit when the application is closing. This ensures that all background processes are properly cleaned up.
---