//! 错误处理模块
//! 
//! 定义了系统监控工具中使用的所有错误类型，提供统一的错误处理机制。

use thiserror::Error;

/// 系统监控工具的主要错误类型
#[derive(Error, Debug)]
pub enum SystemMonitorError {
    /// 系统信息采集错误
    #[error("系统信息采集失败: {0}")]
    SystemInfo(String),

    /// 配置相关错误
    #[error("配置错误: {0}")]
    Config(String),

    /// UI渲染错误
    #[error("UI渲染错误: {0}")]
    Ui(String),

    /// 运行时错误
    #[error("运行时错误: {0}")]
    Runtime(String),

    /// IO错误
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    /// 序列化/反序列化错误
    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    /// 配置解析错误
    #[error("配置解析错误: {0}")]
    ConfigParsing(#[from] config::ConfigError),

    /// 通用错误
    #[error("未知错误: {0}")]
    Other(#[from] anyhow::Error),
}

/// 错误恢复策略
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// 重试操作
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 使用默认值
    UseDefault,
    /// 优雅降级
    Degrade,
    /// 忽略错误
    Ignore,
    /// 终止应用
    Terminate,
}

/// 错误处理器特征
pub trait ErrorHandler {
    /// 处理错误并返回恢复策略
    fn handle_error(&self, error: &SystemMonitorError) -> RecoveryStrategy;
    
    /// 记录错误
    fn log_error(&self, error: &SystemMonitorError, context: &str);
}

/// 默认错误处理器
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &SystemMonitorError) -> RecoveryStrategy {
        match error {
            SystemMonitorError::SystemInfo(_) => RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 1000,
            },
            SystemMonitorError::Config(_) => RecoveryStrategy::UseDefault,
            SystemMonitorError::Ui(_) => RecoveryStrategy::Degrade,
            SystemMonitorError::Runtime(_) => RecoveryStrategy::Terminate,
            SystemMonitorError::Io(_) => RecoveryStrategy::Retry {
                max_attempts: 2,
                delay_ms: 500,
            },
            SystemMonitorError::Serialization(_) => RecoveryStrategy::UseDefault,
            SystemMonitorError::ConfigParsing(_) => RecoveryStrategy::UseDefault,
            SystemMonitorError::Other(_) => RecoveryStrategy::Ignore,
        }
    }

    fn log_error(&self, error: &SystemMonitorError, context: &str) {
        match error {
            SystemMonitorError::Runtime(_) | SystemMonitorError::SystemInfo(_) => {
                log::error!("[{}] 严重错误: {}", context, error);
            }
            SystemMonitorError::Config(_) | SystemMonitorError::ConfigParsing(_) => {
                log::warn!("[{}] 配置警告: {}", context, error);
            }
            _ => {
                log::info!("[{}] 一般错误: {}", context, error);
            }
        }
    }
}

/// 错误恢复执行器
pub struct ErrorRecovery {
    handler: Box<dyn ErrorHandler + Send + Sync>,
}

impl ErrorRecovery {
    /// 创建新的错误恢复执行器
    pub fn new(handler: Box<dyn ErrorHandler + Send + Sync>) -> Self {
        Self { handler }
    }

    /// 创建使用默认处理器的错误恢复执行器
    pub fn default() -> Self {
        Self::new(Box::new(DefaultErrorHandler))
    }

    /// 处理错误并执行恢复策略
    pub async fn handle_with_recovery<T, F, Fut>(
        &self,
        operation: F,
        context: &str,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let _max_attempts = 3;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    self.handler.log_error(&error, context);
                    let strategy = self.handler.handle_error(&error);

                    match strategy {
                        RecoveryStrategy::Retry { max_attempts: max_retry, delay_ms } => {
                            attempts += 1;
                            if attempts >= max_retry {
                                return Err(error);
                            }
                            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
                        }
                        RecoveryStrategy::UseDefault => {
                            log::info!("使用默认值恢复操作: {}", context);
                            return Err(error); // 调用者需要处理默认值逻辑
                        }
                        RecoveryStrategy::Degrade => {
                            log::info!("启用降级模式: {}", context);
                            return Err(error); // 调用者需要处理降级逻辑
                        }
                        RecoveryStrategy::Ignore => {
                            log::info!("忽略错误继续执行: {}", context);
                            return Err(error);
                        }
                        RecoveryStrategy::Terminate => {
                            log::error!("严重错误，终止应用: {}", error);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, SystemMonitorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = SystemMonitorError::SystemInfo("测试错误".to_string());
        assert!(error.to_string().contains("系统信息采集失败"));
    }

    #[test]
    fn test_default_error_handler() {
        let handler = DefaultErrorHandler;
        let error = SystemMonitorError::SystemInfo("测试".to_string());
        
        match handler.handle_error(&error) {
            RecoveryStrategy::Retry { max_attempts, delay_ms } => {
                assert_eq!(max_attempts, 3);
                assert_eq!(delay_ms, 1000);
            }
            _ => panic!("期望重试策略"),
        }
    }
}