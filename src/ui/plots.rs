//! Plot utilities

use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

/// Create a time-series plot
pub fn time_series_plot(
    ui: &mut egui::Ui,
    id: &str,
    data: &[f64],
    color: egui::Color32,
    height: f32,
) {
    let plot = Plot::new(id)
        .height(height)
        .show_axes(true)
        .show_grid(true)
        .allow_zoom(false)
        .allow_drag(false)
        .show_x(false)
        .include_y(0.0);
    
    plot.show(ui, |plot_ui| {
        let points: PlotPoints = data.iter()
            .enumerate()
            .map(|(i, &v)| [i as f64, v])
            .collect();
        
        let line = Line::new(points).color(color).width(1.5);
        plot_ui.line(line);
    });
}

/// Create a spectrum plot (frequency domain)
pub fn spectrum_plot(
    ui: &mut egui::Ui,
    id: &str,
    frequencies: &[f32],
    magnitudes: &[f32],
    height: f32,
) {
    let plot = Plot::new(id)
        .height(height)
        .show_axes(true)
        .show_grid(true)
        .x_axis_label("Frequency (Hz)")
        .y_axis_label("Magnitude (dB)");
    
    plot.show(ui, |plot_ui| {
        let points: PlotPoints = frequencies.iter()
            .zip(magnitudes.iter())
            .map(|(&f, &m)| [f as f64, m as f64])
            .collect();
        
        let line = Line::new(points)
            .color(egui::Color32::LIGHT_BLUE)
            .fill(0.0);
        
        plot_ui.line(line);
    });
}

/// Create a heatmap-style plot
pub fn heatmap(
    ui: &mut egui::Ui,
    data: &[f32],
    width: usize,
    height: usize,
    colormap: impl Fn(f32) -> egui::Color32,
) {
    let available = ui.available_size();
    let cell_w = available.x / width as f32;
    let cell_h = available.y / height as f32;
    
    let (response, painter) = ui.allocate_painter(
        egui::vec2(cell_w * width as f32, cell_h * height as f32),
        egui::Sense::hover(),
    );
    
    let rect = response.rect;
    
    // Find min/max for normalization
    let min = data.iter().fold(f32::MAX, |a, &b| a.min(b));
    let max = data.iter().fold(f32::MIN, |a, &b| a.max(b));
    let range = (max - min).max(1e-6);
    
    for y in 0..height {
        for x in 0..width {
            let value = data[y * width + x];
            let normalized = (value - min) / range;
            let color = colormap(normalized);
            
            let cell_rect = egui::Rect::from_min_size(
                rect.min + egui::vec2(x as f32 * cell_w, y as f32 * cell_h),
                egui::vec2(cell_w, cell_h),
            );
            
            painter.rect_filled(cell_rect, 0.0, color);
        }
    }
}

/// Create a polar plot (for directional sensors)
pub fn polar_plot(
    ui: &mut egui::Ui,
    id: &str,
    angles: &[f32],    // radians
    magnitudes: &[f32],
    size: f32,
) {
    let (response, painter) = ui.allocate_painter(egui::vec2(size, size), egui::Sense::hover());
    let rect = response.rect;
    let center = rect.center();
    let radius = size * 0.4;
    
    // Grid circles
    for r in [0.25, 0.5, 0.75, 1.0] {
        painter.circle_stroke(
            center,
            radius * r,
            egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
        );
    }
    
    // Grid lines
    for i in 0..8 {
        let angle = i as f32 * std::f32::consts::PI / 4.0;
        let end = center + egui::vec2(angle.cos(), angle.sin()) * radius;
        painter.line_segment([center, end], egui::Stroke::new(1.0, egui::Color32::from_gray(60)));
    }
    
    // Find max magnitude for normalization
    let max_mag = magnitudes.iter().fold(0.0f32, |a, &b| a.max(b)).max(1.0);
    
    // Plot data
    let points: Vec<egui::Pos2> = angles.iter()
        .zip(magnitudes.iter())
        .map(|(&a, &m)| {
            let r = (m / max_mag) * radius;
            center + egui::vec2(a.cos() * r, a.sin() * r)
        })
        .collect();
    
    if points.len() > 1 {
        let mut closed_points = points.clone();
        closed_points.push(points[0]);  // Close the loop
        
        painter.add(egui::Shape::line(
            closed_points,
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
        ));
    }
}

/// Waterfall/spectrogram display
pub struct Waterfall {
    data: Vec<Vec<f32>>,
    max_rows: usize,
}

impl Waterfall {
    pub fn new(max_rows: usize) -> Self {
        Self {
            data: Vec::new(),
            max_rows,
        }
    }
    
    pub fn push_row(&mut self, row: Vec<f32>) {
        self.data.push(row);
        if self.data.len() > self.max_rows {
            self.data.remove(0);
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui, colormap: impl Fn(f32) -> egui::Color32) {
        if self.data.is_empty() {
            return;
        }
        
        let width = self.data[0].len();
        let height = self.data.len();
        
        let available = ui.available_size();
        let cell_w = available.x / width as f32;
        let cell_h = (available.y / height as f32).min(4.0);
        
        let (response, painter) = ui.allocate_painter(
            egui::vec2(available.x, cell_h * height as f32),
            egui::Sense::hover(),
        );
        
        let rect = response.rect;
        
        // Find global min/max
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for row in &self.data {
            for &v in row {
                min = min.min(v);
                max = max.max(v);
            }
        }
        let range = (max - min).max(1e-6);
        
        for (y, row) in self.data.iter().enumerate() {
            for (x, &value) in row.iter().enumerate() {
                let normalized = (value - min) / range;
                let color = colormap(normalized);
                
                let cell_rect = egui::Rect::from_min_size(
                    rect.min + egui::vec2(x as f32 * cell_w, y as f32 * cell_h),
                    egui::vec2(cell_w, cell_h),
                );
                
                painter.rect_filled(cell_rect, 0.0, color);
            }
        }
    }
}

/// Multi-line plot for comparing multiple signals
pub fn multi_line_plot(
    ui: &mut egui::Ui,
    id: &str,
    series: &[(&str, &[f64], egui::Color32)],
    height: f32,
) {
    let plot = Plot::new(id)
        .height(height)
        .show_axes(true)
        .show_grid(true)
        .legend(egui_plot::Legend::default());
    
    plot.show(ui, |plot_ui| {
        for (name, data, color) in series {
            let points: PlotPoints = data.iter()
                .enumerate()
                .map(|(i, &v)| [i as f64, v])
                .collect();
            
            let line = Line::new(points)
                .color(*color)
                .name(*name)
                .width(1.5);
            
            plot_ui.line(line);
        }
    });
}
