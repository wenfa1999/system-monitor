//! 系统指标模块
//! 
//! 提供系统性能指标的计算、分析和历史数据管理功能。

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// 性能指标计算器
pub struct MetricsCalculator {
    cpu_history: VecDeque<(Instant, f32)>,
    memory_history: VecDeque<(Instant, f64)>,
    disk_history: VecDeque<(Instant, Vec<f64>)>,
    max_history_size: usize,
    history_duration: Duration,
}

impl MetricsCalculator {
    /// 创建新的指标计算器
    pub fn new(max_history_size: usize, history_duration: Duration) -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(max_history_size),
            memory_history: VecDeque::with_capacity(max_history_size),
            disk_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            history_duration,
        }
    }

    /// 添加CPU使用率数据点
    pub fn add_cpu_data(&mut self, usage: f32) {
        let now = Instant::now();
        self.cpu_history.push_back((now, usage));
        Self::cleanup_old_data_static(&mut self.cpu_history, self.history_duration);
    }

    /// 添加内存使用率数据点
    pub fn add_memory_data(&mut self, usage_percent: f64) {
        let now = Instant::now();
        self.memory_history.push_back((now, usage_percent));
        Self::cleanup_old_data_static(&mut self.memory_history, self.history_duration);
    }

    /// 添加磁盘使用率数据点
    pub fn add_disk_data(&mut self, disk_usages: Vec<f64>) {
        let now = Instant::now();
        self.disk_history.push_back((now, disk_usages));
        Self::cleanup_old_data_static(&mut self.disk_history, self.history_duration);
    }

    /// 清理过期数据
    fn cleanup_old_data<T>(&mut self, history: &mut VecDeque<(Instant, T)>) {
        let _cutoff_time = Instant::now() - self.history_duration;
        Self::cleanup_old_data_static(history, self.history_duration);
    }

    /// 静态方法清理过期数据
    fn cleanup_old_data_static<T>(history: &mut VecDeque<(Instant, T)>, history_duration: Duration) {
        let cutoff_time = Instant::now() - history_duration;
        
        // 移除过期数据
        while let Some((timestamp, _)) = history.front() {
            if *timestamp < cutoff_time {
                history.pop_front();
            } else {
                break;
            }
        }
        
        // 限制历史数据大小
        // 限制历史数据大小（使用默认最大值）
        let max_history_size = 1000; // 默认最大历史记录数
        while history.len() > max_history_size {
            history.pop_front();
        }
    }

    /// 计算CPU使用率统计
    pub fn calculate_cpu_stats(&self) -> CpuStats {
        if self.cpu_history.is_empty() {
            return CpuStats::default();
        }

        let values: Vec<f32> = self.cpu_history.iter().map(|(_, usage)| *usage).collect();
        let sum: f32 = values.iter().sum();
        let count = values.len() as f32;
        
        let average = sum / count;
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        // 计算标准差
        let variance: f32 = values.iter()
            .map(|x| (x - average).powi(2))
            .sum::<f32>() / count;
        let std_deviation = variance.sqrt();

        CpuStats {
            current: values.last().copied().unwrap_or(0.0),
            average,
            min,
            max,
            median,
            std_deviation,
            sample_count: count as usize,
        }
    }

    /// 计算内存使用率统计
    pub fn calculate_memory_stats(&self) -> MemoryStats {
        if self.memory_history.is_empty() {
            return MemoryStats::default();
        }

        let values: Vec<f64> = self.memory_history.iter().map(|(_, usage)| *usage).collect();
        let sum: f64 = values.iter().sum();
        let count = values.len() as f64;
        
        let average = sum / count;
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };

        let variance: f64 = values.iter()
            .map(|x| (x - average).powi(2))
            .sum::<f64>() / count;
        let std_deviation = variance.sqrt();

        MemoryStats {
            current: values.last().copied().unwrap_or(0.0),
            average,
            min,
            max,
            median,
            std_deviation,
            sample_count: count as usize,
        }
    }

    /// 获取CPU历史数据
    pub fn get_cpu_history(&self) -> Vec<f32> {
        self.cpu_history.iter().map(|(_, usage)| *usage).collect()
    }

    /// 获取内存历史数据
    pub fn get_memory_history(&self) -> Vec<f64> {
        self.memory_history.iter().map(|(_, usage)| *usage).collect()
    }

    /// 检测CPU使用率异常
    pub fn detect_cpu_anomalies(&self, threshold_multiplier: f32) -> Vec<CpuAnomaly> {
        let stats = self.calculate_cpu_stats();
        let threshold = stats.average + (stats.std_deviation * threshold_multiplier);
        
        let mut anomalies = Vec::new();
        
        for (timestamp, usage) in &self.cpu_history {
            if *usage > threshold {
                anomalies.push(CpuAnomaly {
                    timestamp: *timestamp,
                    usage: *usage,
                    threshold,
                    severity: if *usage > threshold * 1.5 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    },
                });
            }
        }
        
        anomalies
    }

    /// 预测系统负载趋势
    pub fn predict_load_trend(&self, prediction_window: Duration) -> LoadTrend {
        let cpu_stats = self.calculate_cpu_stats();
        let memory_stats = self.calculate_memory_stats();
        
        // 简单的线性趋势预测
        let cpu_trend = self.calculate_linear_trend(&self.cpu_history);
        let memory_trend = self.calculate_linear_trend(&self.memory_history);
        
        let prediction_seconds = prediction_window.as_secs() as f64;
        let predicted_cpu = cpu_stats.current as f64 + (cpu_trend * prediction_seconds);
        let predicted_memory = memory_stats.current + (memory_trend * prediction_seconds);
        
        LoadTrend {
            cpu_current: cpu_stats.current,
            cpu_predicted: predicted_cpu.max(0.0).min(100.0) as f32,
            cpu_trend: cpu_trend as f32,
            memory_current: memory_stats.current,
            memory_predicted: predicted_memory.max(0.0).min(100.0),
            memory_trend,
            confidence: self.calculate_prediction_confidence(),
        }
    }

    /// 计算线性趋势
    fn calculate_linear_trend<T>(&self, history: &VecDeque<(Instant, T)>) -> f64 
    where
        T: Copy + Into<f64>,
    {
        if history.len() < 2 {
            return 0.0;
        }

        let base_time = history.front().unwrap().0;
        let points: Vec<(f64, f64)> = history.iter()
            .map(|(timestamp, value)| {
                let x = timestamp.duration_since(base_time).as_secs_f64();
                let y = (*value).into();
                (x, y)
            })
            .collect();

        // 简单线性回归
        let n = points.len() as f64;
        let sum_x: f64 = points.iter().map(|(x, _)| *x).sum();
        let sum_y: f64 = points.iter().map(|(_, y)| *y).sum();
        let sum_xy: f64 = points.iter().map(|(x, y)| x * y).sum();
        let sum_x2: f64 = points.iter().map(|(x, _)| x * x).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < f64::EPSILON {
            return 0.0;
        }

        (n * sum_xy - sum_x * sum_y) / denominator
    }

    /// 计算预测置信度
    fn calculate_prediction_confidence(&self) -> f64 {
        let cpu_stats = self.calculate_cpu_stats();
        let memory_stats = self.calculate_memory_stats();
        
        // 基于数据稳定性计算置信度
        let cpu_stability = 1.0 - (cpu_stats.std_deviation / 100.0).min(1.0) as f64;
        let memory_stability = 1.0 - (memory_stats.std_deviation / 100.0).min(1.0);
        let sample_confidence = (self.cpu_history.len().min(100) as f64) / 100.0;
        
        (cpu_stability + memory_stability + sample_confidence) / 3.0
    }
}

/// CPU统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStats {
    pub current: f32,
    pub average: f32,
    pub min: f32,
    pub max: f32,
    pub median: f32,
    pub std_deviation: f32,
    pub sample_count: usize,
}

impl Default for CpuStats {
    fn default() -> Self {
        Self {
            current: 0.0,
            average: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            std_deviation: 0.0,
            sample_count: 0,
        }
    }
}

/// 内存统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current: f64,
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub std_deviation: f64,
    pub sample_count: usize,
}

impl Default for MemoryStats {
    fn default() -> Self {
        Self {
            current: 0.0,
            average: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            std_deviation: 0.0,
            sample_count: 0,
        }
    }
}

/// CPU异常检测结果
#[derive(Debug, Clone)]
pub struct CpuAnomaly {
    pub timestamp: Instant,
    pub usage: f32,
    pub threshold: f32,
    pub severity: AnomalySeverity,
}

/// 异常严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 负载趋势预测
#[derive(Debug, Clone)]
pub struct LoadTrend {
    pub cpu_current: f32,
    pub cpu_predicted: f32,
    pub cpu_trend: f32,
    pub memory_current: f64,
    pub memory_predicted: f64,
    pub memory_trend: f64,
    pub confidence: f64,
}

/// 性能基准测试
pub struct PerformanceBenchmark {
    baseline_cpu: f32,
    baseline_memory: f64,
    baseline_timestamp: Instant,
}

impl PerformanceBenchmark {
    /// 创建新的性能基准
    pub fn new(cpu_usage: f32, memory_usage: f64) -> Self {
        Self {
            baseline_cpu: cpu_usage,
            baseline_memory: memory_usage,
            baseline_timestamp: Instant::now(),
        }
    }

    /// 计算性能变化
    pub fn calculate_performance_change(&self, current_cpu: f32, current_memory: f64) -> PerformanceChange {
        let cpu_change = ((current_cpu - self.baseline_cpu) / self.baseline_cpu) * 100.0;
        let memory_change = ((current_memory - self.baseline_memory) / self.baseline_memory) * 100.0;
        let time_elapsed = self.baseline_timestamp.elapsed();

        PerformanceChange {
            cpu_change_percent: cpu_change,
            memory_change_percent: memory_change,
            time_elapsed,
            overall_trend: if cpu_change > 10.0 || memory_change > 10.0 {
                TrendDirection::Increasing
            } else if cpu_change < -10.0 || memory_change < -10.0 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            },
        }
    }
}

/// 性能变化信息
#[derive(Debug, Clone)]
pub struct PerformanceChange {
    pub cpu_change_percent: f32,
    pub memory_change_percent: f64,
    pub time_elapsed: Duration,
    pub overall_trend: TrendDirection,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_calculator() {
        let mut calculator = MetricsCalculator::new(100, Duration::from_secs(3600));
        
        // 添加测试数据
        calculator.add_cpu_data(25.0);
        calculator.add_cpu_data(30.0);
        calculator.add_cpu_data(35.0);
        
        let stats = calculator.calculate_cpu_stats();
        assert_eq!(stats.sample_count, 3);
        assert_eq!(stats.average, 30.0);
        assert_eq!(stats.min, 25.0);
        assert_eq!(stats.max, 35.0);
    }

    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = crate::utils::MathUtils::moving_average(&values, 3);
        assert_eq!(avg.len(), 5);
        assert_eq!(avg[2], 2.0); // (1+2+3)/3
        assert_eq!(avg[4], 4.0); // (3+4+5)/3
    }
}