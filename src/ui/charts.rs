//! 图表组件模块
//! 
//! 提供各种数据可视化图表组件。

use eframe::egui;
use std::collections::VecDeque;

/// 简单的线性图表组件
pub struct LineChart {
    data: VecDeque<f32>,
    max_points: usize,
    min_value: f32,
    max_value: f32,
    color: egui::Color32,
    fill_color: Option<egui::Color32>,
}

impl LineChart {
    /// 创建新的线性图表
    pub fn new(max_points: usize, color: egui::Color32) -> Self {
        Self {
            data: VecDeque::with_capacity(max_points),
            max_points,
            min_value: 0.0,
            max_value: 100.0,
            color,
            fill_color: None,
        }
    }

    /// 设置值范围
    pub fn with_range(mut self, min: f32, max: f32) -> Self {
        self.min_value = min;
        self.max_value = max;
        self
    }

    /// 设置填充颜色
    pub fn with_fill(mut self, fill_color: egui::Color32) -> Self {
        self.fill_color = Some(fill_color);
        self
    }

    /// 添加数据点
    pub fn add_point(&mut self, value: f32) {
        if self.data.len() >= self.max_points {
            self.data.pop_front();
        }
        self.data.push_back(value);
    }

    /// 设置数据
    pub fn set_data(&mut self, data: Vec<f32>) {
        self.data.clear();
        for value in data.into_iter().take(self.max_points) {
            self.data.push_back(value);
        }
    }

    /// 渲染图表
    pub fn render(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;

        if self.data.is_empty() {
            // 绘制空图表
            painter.rect_stroke(rect, 0.0, egui::Stroke::new(1.0, ui.visuals().weak_text_color()));
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "无数据",
                egui::FontId::default(),
                ui.visuals().weak_text_color(),
            );
            return response;
        }

        // 绘制背景
        painter.rect_filled(rect, 2.0, ui.visuals().extreme_bg_color);
        painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, ui.visuals().weak_text_color()));

        // 计算点的位置
        let points: Vec<egui::Pos2> = self.data
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let x = rect.left() + (i as f32 / (self.max_points - 1).max(1) as f32) * rect.width();
                let normalized_value = (value - self.min_value) / (self.max_value - self.min_value);
                let y = rect.bottom() - normalized_value.clamp(0.0, 1.0) * rect.height();
                egui::Pos2::new(x, y)
            })
            .collect();

        // 绘制填充区域
        if let Some(fill_color) = self.fill_color {
            if points.len() > 1 {
                let mut fill_points = points.clone();
                fill_points.push(egui::Pos2::new(points.last().unwrap().x, rect.bottom()));
                fill_points.push(egui::Pos2::new(points.first().unwrap().x, rect.bottom()));
                
                painter.add(egui::Shape::convex_polygon(
                    fill_points,
                    fill_color,
                    egui::Stroke::NONE,
                ));
            }
        }

        // 绘制线条
        if points.len() > 1 {
            painter.add(egui::Shape::line(
                points.clone(),
                egui::Stroke::new(2.0, self.color),
            ));
        }

        // 绘制数据点
        for point in &points {
            painter.circle_filled(*point, 2.0, self.color);
        }

        // 绘制网格线（可选）
        self.draw_grid(&painter, rect, ui);

        // 绘制数值标签
        self.draw_labels(&painter, rect, ui);

        response
    }

    /// 绘制网格线
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect, ui: &egui::Ui) {
        let grid_color = ui.visuals().weak_text_color().gamma_multiply(0.3);
        let stroke = egui::Stroke::new(0.5, grid_color);

        // 水平网格线
        for i in 1..5 {
            let y = rect.top() + (i as f32 / 5.0) * rect.height();
            painter.line_segment(
                [egui::Pos2::new(rect.left(), y), egui::Pos2::new(rect.right(), y)],
                stroke,
            );
        }

        // 垂直网格线
        for i in 1..10 {
            let x = rect.left() + (i as f32 / 10.0) * rect.width();
            painter.line_segment(
                [egui::Pos2::new(x, rect.top()), egui::Pos2::new(x, rect.bottom())],
                stroke,
            );
        }
    }

    /// 绘制数值标签
    fn draw_labels(&self, painter: &egui::Painter, rect: egui::Rect, ui: &egui::Ui) {
        let text_color = ui.visuals().text_color();
        let font_id = egui::FontId::monospace(10.0);

        // Y轴标签
        for i in 0..=4 {
            let value = self.max_value - (i as f32 / 4.0) * (self.max_value - self.min_value);
            let y = rect.top() + (i as f32 / 4.0) * rect.height();
            let text = if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                format!("{:.1}", value)
            };
            
            painter.text(
                egui::Pos2::new(rect.left() - 5.0, y),
                egui::Align2::RIGHT_CENTER,
                text,
                font_id.clone(),
                text_color,
            );
        }

        // 当前值显示
        if let Some(&last_value) = self.data.back() {
            let text = format!("{:.1}", last_value);
            painter.text(
                egui::Pos2::new(rect.right() + 5.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                text,
                font_id,
                self.color,
            );
        }
    }

    /// 获取当前数据
    pub fn get_data(&self) -> Vec<f32> {
        self.data.iter().cloned().collect()
    }

    /// 清空数据
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// 环形进度图表
pub struct DonutChart {
    value: f32,
    max_value: f32,
    color: egui::Color32,
    background_color: egui::Color32,
    thickness: f32,
}

impl DonutChart {
    /// 创建新的环形图表
    pub fn new(value: f32, max_value: f32, color: egui::Color32) -> Self {
        Self {
            value,
            max_value,
            color,
            background_color: egui::Color32::from_gray(50),
            thickness: 8.0,
        }
    }

    /// 设置背景颜色
    pub fn with_background_color(mut self, color: egui::Color32) -> Self {
        self.background_color = color;
        self
    }

    /// 设置线条粗细
    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    /// 更新数值
    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    /// 渲染环形图表
    pub fn render(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let radius = (rect.width().min(rect.height()) / 2.0) - self.thickness;

        // 绘制背景圆环
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(self.thickness, self.background_color),
        );

        // 计算进度角度
        let progress = (self.value / self.max_value).clamp(0.0, 1.0);
        let angle = progress * 2.0 * std::f32::consts::PI;

        // 绘制进度圆弧
        if progress > 0.0 {
            self.draw_arc(&painter, center, radius, angle);
        }

        // 绘制中心文本
        let percentage = (progress * 100.0) as i32;
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{}%", percentage),
            egui::FontId::proportional(16.0),
            ui.visuals().text_color(),
        );

        response
    }

    /// 绘制圆弧
    fn draw_arc(&self, painter: &egui::Painter, center: egui::Pos2, radius: f32, angle: f32) {
        let segments = (angle * 20.0) as usize + 1;
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = (i as f32 / segments as f32) * angle;
            let x = center.x + radius * (t - std::f32::consts::PI / 2.0).cos();
            let y = center.y + radius * (t - std::f32::consts::PI / 2.0).sin();
            points.push(egui::Pos2::new(x, y));
        }

        if points.len() > 1 {
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(self.thickness, self.color),
            ));
        }
    }
}

/// 简单的柱状图
pub struct BarChart {
    data: Vec<(String, f32)>,
    max_value: f32,
    bar_color: egui::Color32,
}

impl BarChart {
    /// 创建新的柱状图
    pub fn new(data: Vec<(String, f32)>, bar_color: egui::Color32) -> Self {
        let max_value = data.iter().map(|(_, v)| *v).fold(0.0, f32::max);
        Self {
            data,
            max_value,
            bar_color,
        }
    }

    /// 渲染柱状图
    pub fn render(&self, ui: &mut egui::Ui, size: egui::Vec2) -> egui::Response {
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;

        if self.data.is_empty() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "无数据",
                egui::FontId::default(),
                ui.visuals().weak_text_color(),
            );
            return response;
        }

        let bar_width = rect.width() / self.data.len() as f32 * 0.8;
        let bar_spacing = rect.width() / self.data.len() as f32 * 0.2;

        for (i, (label, value)) in self.data.iter().enumerate() {
            let x = rect.left() + (i as f32 + 0.5) * (bar_width + bar_spacing);
            let height = (value / self.max_value) * rect.height() * 0.8;
            let bar_rect = egui::Rect::from_min_size(
                egui::Pos2::new(x - bar_width / 2.0, rect.bottom() - height),
                egui::Vec2::new(bar_width, height),
            );

            // 绘制柱子
            painter.rect_filled(bar_rect, 2.0, self.bar_color);
            painter.rect_stroke(bar_rect, 2.0, egui::Stroke::new(1.0, ui.visuals().text_color()));

            // 绘制标签
            painter.text(
                egui::Pos2::new(x, rect.bottom() + 5.0),
                egui::Align2::CENTER_TOP,
                label,
                egui::FontId::monospace(10.0),
                ui.visuals().text_color(),
            );

            // 绘制数值
            painter.text(
                egui::Pos2::new(x, bar_rect.top() - 5.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{:.1}", value),
                egui::FontId::monospace(10.0),
                ui.visuals().text_color(),
            );
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_chart_creation() {
        let chart = LineChart::new(100, egui::Color32::BLUE);
        assert_eq!(chart.max_points, 100);
        assert_eq!(chart.data.len(), 0);
    }

    #[test]
    fn test_line_chart_add_point() {
        let mut chart = LineChart::new(3, egui::Color32::BLUE);
        chart.add_point(10.0);
        chart.add_point(20.0);
        chart.add_point(30.0);
        chart.add_point(40.0); // 应该移除第一个点

        assert_eq!(chart.data.len(), 3);
        assert_eq!(chart.data[0], 20.0);
        assert_eq!(chart.data[2], 40.0);
    }

    #[test]
    fn test_donut_chart_creation() {
        let chart = DonutChart::new(75.0, 100.0, egui::Color32::GREEN);
        assert_eq!(chart.value, 75.0);
        assert_eq!(chart.max_value, 100.0);
    }
}