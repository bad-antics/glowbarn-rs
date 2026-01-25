// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Custom UI widgets

use eframe::egui;

/// Status indicator (green/red dot with label)
pub fn status_indicator(ui: &mut egui::Ui, online: bool, label: &str) {
    ui.horizontal(|ui| {
        let color = if online {
            egui::Color32::GREEN
        } else {
            egui::Color32::RED
        };
        ui.colored_label(color, "●");
        ui.label(label);
    });
}

/// Severity badge
pub fn severity_badge(ui: &mut egui::Ui, severity: &crate::detection::Severity) {
    let (text, bg_color) = match severity {
        crate::detection::Severity::Critical => ("CRITICAL", egui::Color32::from_rgb(200, 0, 0)),
        crate::detection::Severity::High => ("HIGH", egui::Color32::from_rgb(200, 100, 0)),
        crate::detection::Severity::Medium => ("MEDIUM", egui::Color32::from_rgb(200, 200, 0)),
        crate::detection::Severity::Low => ("LOW", egui::Color32::from_rgb(0, 150, 0)),
    };
    
    let text_color = egui::Color32::WHITE;
    
    egui::Frame::none()
        .fill(bg_color)
        .inner_margin(egui::Margin::symmetric(6.0, 2.0))
        .rounding(4.0)
        .show(ui, |ui| {
            ui.colored_label(text_color, text);
        });
}

/// Gauge widget
pub struct Gauge {
    value: f32,
    min: f32,
    max: f32,
    label: String,
    unit: String,
}

impl Gauge {
    pub fn new(value: f32, min: f32, max: f32) -> Self {
        Self {
            value,
            min,
            max,
            label: String::new(),
            unit: String::new(),
        }
    }
    
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }
    
    pub fn show(self, ui: &mut egui::Ui) {
        let size = egui::vec2(100.0, 100.0);
        let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
        let rect = response.rect;
        let center = rect.center();
        let radius = rect.width().min(rect.height()) * 0.4;
        
        // Background arc
        painter.circle_stroke(
            center,
            radius,
            egui::Stroke::new(8.0, egui::Color32::from_gray(60)),
        );
        
        // Value arc
        let ratio = ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        let color = if ratio < 0.5 {
            egui::Color32::GREEN
        } else if ratio < 0.8 {
            egui::Color32::YELLOW
        } else {
            egui::Color32::RED
        };
        
        // Draw arc segments
        let segments = (ratio * 32.0) as usize;
        for i in 0..segments {
            let angle1 = std::f32::consts::PI * (0.75 + (i as f32 / 32.0) * 1.5);
            let angle2 = std::f32::consts::PI * (0.75 + ((i + 1) as f32 / 32.0) * 1.5);
            
            let p1 = center + egui::vec2(angle1.cos(), angle1.sin()) * radius;
            let p2 = center + egui::vec2(angle2.cos(), angle2.sin()) * radius;
            
            painter.line_segment([p1, p2], egui::Stroke::new(8.0, color));
        }
        
        // Value text
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            format!("{:.1}{}", self.value, self.unit),
            egui::FontId::proportional(16.0),
            egui::Color32::WHITE,
        );
        
        // Label
        if !self.label.is_empty() {
            painter.text(
                center + egui::vec2(0.0, radius + 15.0),
                egui::Align2::CENTER_CENTER,
                &self.label,
                egui::FontId::proportional(12.0),
                egui::Color32::GRAY,
            );
        }
    }
}

/// LED indicator
pub fn led(ui: &mut egui::Ui, on: bool, size: f32) {
    let color = if on {
        egui::Color32::from_rgb(0, 255, 0)
    } else {
        egui::Color32::from_rgb(50, 50, 50)
    };
    
    let (response, painter) = ui.allocate_painter(egui::vec2(size, size), egui::Sense::hover());
    let center = response.rect.center();
    
    // Glow effect
    if on {
        painter.circle_filled(center, size * 0.4, egui::Color32::from_rgba_unmultiplied(0, 255, 0, 50));
    }
    
    // LED body
    painter.circle_filled(center, size * 0.3, color);
    
    // Highlight
    painter.circle_filled(
        center + egui::vec2(-size * 0.1, -size * 0.1),
        size * 0.1,
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100),
    );
}

/// Horizontal level meter
pub fn level_meter(ui: &mut egui::Ui, value: f32, min: f32, max: f32, width: f32) {
    let height = 16.0;
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = response.rect;
    
    // Background
    painter.rect_filled(rect, 2.0, egui::Color32::from_gray(40));
    
    // Level
    let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);
    let level_rect = egui::Rect::from_min_size(
        rect.min,
        egui::vec2(rect.width() * ratio, rect.height()),
    );
    
    let color = if ratio < 0.6 {
        egui::Color32::GREEN
    } else if ratio < 0.85 {
        egui::Color32::YELLOW
    } else {
        egui::Color32::RED
    };
    
    painter.rect_filled(level_rect, 2.0, color);
    
    // Border
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_gray(100)));
}

/// Sparkline (minimal inline chart)
pub fn sparkline(ui: &mut egui::Ui, data: &[f64], width: f32, height: f32) {
    if data.is_empty() {
        return;
    }
    
    let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
    let rect = response.rect;
    
    let min = data.iter().fold(f64::MAX, |a, &b| a.min(b));
    let max = data.iter().fold(f64::MIN, |a, &b| a.max(b));
    let range = (max - min).max(1e-10);
    
    let step = rect.width() / (data.len() - 1).max(1) as f32;
    
    let points: Vec<egui::Pos2> = data.iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = rect.left() + i as f32 * step;
            let y = rect.bottom() - (((v - min) / range) as f32 * rect.height());
            egui::pos2(x, y)
        })
        .collect();
    
    painter.add(egui::Shape::line(
        points,
        egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
    ));
}

/// Alert card
pub fn alert_card(
    ui: &mut egui::Ui,
    title: &str,
    message: &str,
    severity: &crate::detection::Severity,
) {
    let bg_color = match severity {
        crate::detection::Severity::Critical => egui::Color32::from_rgb(80, 0, 0),
        crate::detection::Severity::High => egui::Color32::from_rgb(80, 40, 0),
        crate::detection::Severity::Medium => egui::Color32::from_rgb(80, 80, 0),
        crate::detection::Severity::Low => egui::Color32::from_rgb(0, 40, 0),
    };
    
    egui::Frame::none()
        .fill(bg_color)
        .inner_margin(8.0)
        .rounding(4.0)
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(100)))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                severity_badge(ui, severity);
                ui.strong(title);
            });
            ui.label(message);
        });
}

/// Sensor card widget
pub fn sensor_card(
    ui: &mut egui::Ui,
    name: &str,
    sensor_type: &str,
    value: f64,
    unit: &str,
    online: bool,
) {
    egui::Frame::none()
        .fill(egui::Color32::from_gray(30))
        .inner_margin(8.0)
        .rounding(4.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                led(ui, online, 12.0);
                ui.strong(name);
            });
            
            ui.small(sensor_type);
            
            ui.horizontal(|ui| {
                ui.heading(format!("{:.2}", value));
                ui.label(unit);
            });
        });
}
