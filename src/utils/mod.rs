//! 工具模块
//! 
//! 提供各种实用工具函数和助手。

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 时间工具
pub struct TimeUtils;

impl TimeUtils {
    /// 获取当前Unix时间戳
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }

    /// 格式化持续时间为人类可读格式
    pub fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if days > 0 {
            format!("{}天 {}小时 {}分钟", days, hours, minutes)
        } else if hours > 0 {
            format!("{}小时 {}分钟", hours, minutes)
        } else if minutes > 0 {
            format!("{}分钟 {}秒", minutes, seconds)
        } else {
            format!("{}秒", seconds)
        }
    }

    /// 格式化时间戳为本地时间字符串
    pub fn format_timestamp(timestamp: u64) -> String {
        let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        let local_datetime = datetime.with_timezone(&chrono::Local);
        local_datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

/// 数学工具
pub struct MathUtils;

impl MathUtils {
    /// 计算百分比
    pub fn percentage(value: f64, total: f64) -> f64 {
        if total == 0.0 {
            0.0
        } else {
            (value / total) * 100.0
        }
    }

    /// 将值限制在指定范围内
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// 计算移动平均值
    pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        if values.is_empty() || window_size == 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        for i in 0..values.len() {
            let start = if i >= window_size - 1 { i - window_size + 1 } else { 0 };
            let end = i + 1;
            let window = &values[start..end];
            let average = window.iter().sum::<f64>() / window.len() as f64;
            result.push(average);
        }
        result
    }

    /// 计算标准差
    pub fn standard_deviation(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }
}

/// 字符串工具
pub struct StringUtils;

impl StringUtils {
    /// 截断字符串到指定长度
    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }

    /// 格式化文件大小
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
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

    /// 格式化数字为千分位格式
    pub fn format_number(num: u64) -> String {
        let num_str = num.to_string();
        let mut result = String::new();
        
        for (i, c) in num_str.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        
        result.chars().rev().collect()
    }
}

/// 系统工具
pub struct SystemUtils;

impl SystemUtils {
    /// 检查是否为Windows系统
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// 检查是否为Linux系统
    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    /// 检查是否为macOS系统
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// 获取操作系统名称
    pub fn os_name() -> &'static str {
        if Self::is_windows() {
            "Windows"
        } else if Self::is_linux() {
            "Linux"
        } else if Self::is_macos() {
            "macOS"
        } else {
            "Unknown"
        }
    }

    /// 获取CPU核心数
    pub fn cpu_count() -> usize {
        num_cpus::get()
    }
}

/// 颜色工具
pub struct ColorUtils;

impl ColorUtils {
    /// 根据值获取渐变颜色
    pub fn gradient_color(value: f32, min_val: f32, max_val: f32) -> [f32; 3] {
        let normalized = ((value - min_val) / (max_val - min_val)).clamp(0.0, 1.0);
        
        if normalized < 0.5 {
            // 绿色到黄色
            let t = normalized * 2.0;
            [t, 1.0, 0.0]
        } else {
            // 黄色到红色
            let t = (normalized - 0.5) * 2.0;
            [1.0, 1.0 - t, 0.0]
        }
    }

    /// 获取使用率对应的颜色
    pub fn usage_color(usage_percent: f64) -> [f32; 3] {
        match usage_percent {
            x if x < 30.0 => [0.3, 0.8, 0.3],  // 绿色
            x if x < 60.0 => [0.8, 0.8, 0.3],  // 黄色
            x if x < 80.0 => [0.9, 0.6, 0.2],  // 橙色
            _ => [0.9, 0.3, 0.3],               // 红色
        }
    }

    /// RGB转换为HSV
    pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let s = if max == 0.0 { 0.0 } else { delta / max };
        let v = max;

        (h, s, v)
    }

    /// HSV转换为RGB
    pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = match h as i32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            300..=359 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        (r_prime + m, g_prime + m, b_prime + m)
    }
}

/// 验证工具
pub struct ValidationUtils;

impl ValidationUtils {
    /// 验证端口号
    pub fn is_valid_port(port: u16) -> bool {
        port > 0 && port <= 65535
    }

    /// 验证IP地址格式（简单验证）
    pub fn is_valid_ip(ip: &str) -> bool {
        ip.split('.')
            .filter_map(|s| s.parse::<u8>().ok())
            .count() == 4
    }

    /// 验证文件路径
    pub fn is_valid_path(path: &str) -> bool {
        !path.is_empty() && !path.contains('\0')
    }

    /// 验证数值范围
    pub fn is_in_range<T: PartialOrd>(value: T, min: T, max: T) -> bool {
        value >= min && value <= max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_utils() {
        let timestamp = TimeUtils::current_timestamp();
        assert!(timestamp > 0);
        
        let duration = Duration::from_secs(3661);
        let formatted = TimeUtils::format_duration(duration);
        assert!(formatted.contains("1小时"));
    }

    #[test]
    fn test_math_utils() {
        assert_eq!(MathUtils::percentage(25.0, 100.0), 25.0);
        assert_eq!(MathUtils::clamp(15, 10, 20), 15);
        assert_eq!(MathUtils::clamp(5, 10, 20), 10);
        assert_eq!(MathUtils::clamp(25, 10, 20), 20);
    }

    #[test]
    fn test_string_utils() {
        assert_eq!(StringUtils::format_file_size(1024), "1.0 KB");
        assert_eq!(StringUtils::format_file_size(1048576), "1.0 MB");
        assert_eq!(StringUtils::truncate("Hello World", 5), "He...");
        assert_eq!(StringUtils::format_number(1234567), "1,234,567");
    }

    #[test]
    fn test_color_utils() {
        let color = ColorUtils::usage_color(25.0);
        assert_eq!(color, [0.3, 0.8, 0.3]); // 绿色
        
        let color = ColorUtils::usage_color(85.0);
        assert_eq!(color, [0.9, 0.3, 0.3]); // 红色
    }

    #[test]
    fn test_validation_utils() {
        assert!(ValidationUtils::is_valid_port(8080));
        assert!(!ValidationUtils::is_valid_port(0));
        assert!(ValidationUtils::is_valid_ip("192.168.1.1"));
        assert!(!ValidationUtils::is_valid_ip("256.1.1.1"));
        assert!(ValidationUtils::is_in_range(15, 10, 20));
        assert!(!ValidationUtils::is_in_range(25, 10, 20));
    }

    #[test]
    fn test_moving_average() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let avg = MathUtils::moving_average(&values, 3);
        assert_eq!(avg.len(), 5);
        assert_eq!(avg[2], 2.0); // (1+2+3)/3
        assert_eq!(avg[4], 4.0); // (3+4+5)/3
    }
}