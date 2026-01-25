//! UI panels

use eframe::egui;
use crate::sensors::SensorType;
use crate::detection::{DetectionType, Severity};
use super::{GuiState, ThermalData, SpectrumData};
use super::plots::*;
use super::widgets::*;

/// Sensor list panel
pub struct SensorPanel {
    search_filter: String,
}

impl SensorPanel {
    pub fn new() -> Self {
        Self {
            search_filter: String::new(),
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, state: &mut GuiState) {
        ui.heading("🔌 Sensors");
        ui.separator();
        
        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_filter);
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Demo sensors
            let sensors = [
                ("EMF-001", SensorType::EMFProbe, true),
                ("Thermal-001", SensorType::ThermalArray, true),
                ("Audio-001", SensorType::FullSpectrum, true),
                ("Seismic-001", SensorType::Geophone, true),
                ("Geiger-001", SensorType::GeigerCounter, true),
                ("RF-001", SensorType::SDRReceiver, true),
                ("QRNG-001", SensorType::QRNG, true),
                ("Laser-001", SensorType::LaserGrid, true),
                ("FluxGate-001", SensorType::FluxGate, true),
                ("Ion-001", SensorType::IonCounter, false),
                ("UV-001", SensorType::UVSensor, true),
                ("Infra-001", SensorType::Infrasound, true),
                ("Ultra-001", SensorType::Ultrasonic, true),
                ("Static-001", SensorType::StaticMeter, true),
            ];
            
            for (id, sensor_type, online) in sensors {
                if !self.search_filter.is_empty() 
                    && !id.to_lowercase().contains(&self.search_filter.to_lowercase()) {
                    continue;
                }
                
                let selected = state.selected_sensor.as_deref() == Some(id);
                
                ui.horizontal(|ui| {
                    // Status dot
                    let color = if online {
                        egui::Color32::GREEN
                    } else {
                        egui::Color32::RED
                    };
                    ui.colored_label(color, "●");
                    
                    // Sensor button
                    let response = ui.selectable_label(selected, format!("{:?}", sensor_type));
                    if response.clicked() {
                        state.selected_sensor = Some(id.to_string());
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.small(id);
                    });
                });
            }
        });
        
        ui.separator();
        
        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("▶ Start All").clicked() {
                // Start all sensors
            }
            if ui.button("⏹ Stop All").clicked() {
                // Stop all sensors
            }
        });
    }
}

/// Waveform display panel
pub struct WaveformPanel {
    show_grid: bool,
    auto_scale: bool,
}

impl WaveformPanel {
    pub fn new() -> Self {
        Self {
            show_grid: true,
            auto_scale: true,
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui, state: &GuiState) {
        ui.heading("📈 Real-time Waveforms");
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (sensor_id, data) in &state.waveforms {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(sensor_id);
                        
                        if let Some(last) = data.last() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.monospace(format!("{:.2}", last));
                            });
                        }
                    });
                    
                    // Plot
                    let height = 80.0;
                    let width = ui.available_width();
                    
                    let plot = egui_plot::Plot::new(format!("waveform_{}", sensor_id))
                        .height(height)
                        .width(width)
                        .show_axes(true)
                        .show_grid(self.show_grid)
                        .allow_zoom(false)
                        .allow_drag(false)
                        .include_y(0.0);
                    
                    plot.show(ui, |plot_ui| {
                        let points: egui_plot::PlotPoints = data.iter()
                            .enumerate()
                            .map(|(i, &v)| [i as f64, v])
                            .collect();
                        
                        let line = egui_plot::Line::new(points)
                            .color(get_sensor_color(sensor_id))
                            .width(1.5);
                        
                        plot_ui.line(line);
                    });
                });
            }
        });
    }
}

/// Thermal imaging panel
pub struct ThermalPanel {
    colormap: Colormap,
    show_temps: bool,
}

impl ThermalPanel {
    pub fn new() -> Self {
        Self {
            colormap: Colormap::Inferno,
            show_temps: true,
        }
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, state: &GuiState) {
        ui.heading("🌡️ Thermal");
        
        if let Some(ref thermal) = state.thermal_data {
            // Temperature range
            ui.horizontal(|ui| {
                ui.small(format!("Min: {:.1}°C", thermal.min_temp));
                ui.small(format!("Max: {:.1}°C", thermal.max_temp));
            });
            
            // Draw thermal grid
            let available = ui.available_size();
            let cell_w = (available.x / thermal.width as f32).min(12.0);
            let cell_h = (available.y / thermal.height as f32).min(12.0);
            
            let (response, painter) = ui.allocate_painter(
                egui::vec2(cell_w * thermal.width as f32, cell_h * thermal.height as f32),
                egui::Sense::hover(),
            );
            
            let rect = response.rect;
            
            for y in 0..thermal.height {
                for x in 0..thermal.width {
                    let temp = thermal.data[y * thermal.width + x];
                    let normalized = (temp - thermal.min_temp) / (thermal.max_temp - thermal.min_temp);
                    let color = self.colormap.to_color(normalized);
                    
                    let cell_rect = egui::Rect::from_min_size(
                        rect.min + egui::vec2(x as f32 * cell_w, y as f32 * cell_h),
                        egui::vec2(cell_w, cell_h),
                    );
                    
                    painter.rect_filled(cell_rect, 0.0, color);
                }
            }
            
            // Show temperature on hover
            if let Some(pos) = response.hover_pos() {
                let local = pos - rect.min;
                let x = (local.x / cell_w) as usize;
                let y = (local.y / cell_h) as usize;
                
                if x < thermal.width && y < thermal.height {
                    let temp = thermal.data[y * thermal.width + x];
                    egui::show_tooltip_at_pointer(ui.ctx(), egui::Id::new("thermal_tooltip"), |ui| {
                        ui.label(format!("{:.1}°C", temp));
                    });
                }
            }
            
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No thermal data");
            });
        }
    }
}

/// Spectrum analyzer panel
pub struct SpectrumPanel {
    log_scale: bool,
}

impl SpectrumPanel {
    pub fn new() -> Self {
        Self {
            log_scale: true,
        }
    }
    
    pub fn show(&self, ui: &mut egui::Ui, state: &GuiState) {
        ui.heading("📊 Spectrum Analyzer");
        
        if let Some(ref spectrum) = state.spectrum_data {
            ui.horizontal(|ui| {
                ui.small(format!("Peak: {:.0} Hz", spectrum.peak_freq));
            });
            
            let plot = egui_plot::Plot::new("spectrum")
                .height(150.0)
                .show_axes(true)
                .show_grid(true)
                .allow_zoom(true)
                .allow_drag(true);
            
            plot.show(ui, |plot_ui| {
                let points: egui_plot::PlotPoints = spectrum.frequencies.iter()
                    .zip(spectrum.magnitudes.iter())
                    .map(|(&f, &m)| [f as f64, m as f64])
                    .collect();
                
                let line = egui_plot::Line::new(points)
                    .color(egui::Color32::LIGHT_BLUE)
                    .fill(0.0);
                
                plot_ui.line(line);
            });
        } else {
            ui.centered_and_justified(|ui| {
                ui.label("No spectrum data");
            });
        }
    }
}

/// Detection events panel
pub struct DetectionPanel;

impl DetectionPanel {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&self, ui: &mut egui::Ui, state: &mut GuiState) {
        ui.heading("⚠️ Detections");
        ui.separator();
        
        // Summary
        let critical = state.detections.iter().filter(|d| d.severity == Severity::Critical).count();
        let high = state.detections.iter().filter(|d| d.severity == Severity::High).count();
        
        ui.horizontal(|ui| {
            if critical > 0 {
                ui.colored_label(egui::Color32::RED, format!("🔴 {} Critical", critical));
            }
            if high > 0 {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), format!("🟠 {} High", high));
            }
        });
        
        ui.separator();
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            for detection in state.detections.iter().rev().take(50) {
                ui.group(|ui| {
                    let (icon, color) = match detection.severity {
                        Severity::Critical => ("🔴", egui::Color32::RED),
                        Severity::High => ("🟠", egui::Color32::from_rgb(255, 165, 0)),
                        Severity::Medium => ("🟡", egui::Color32::YELLOW),
                        Severity::Low => ("🟢", egui::Color32::GREEN),
                    };
                    
                    ui.horizontal(|ui| {
                        ui.label(icon);
                        ui.colored_label(color, format!("{:?}", detection.detection_type));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.small(format!("Confidence: {:.0}%", detection.confidence * 100.0));
                        ui.small(format!("| {}", detection.timestamp.format("%H:%M:%S")));
                    });
                    
                    if detection.entropy_deviation > 0.1 {
                        ui.small(format!("Entropy dev: {:.2}", detection.entropy_deviation));
                    }
                });
            }
        });
        
        ui.separator();
        
        if ui.button("Clear All").clicked() {
            state.detections.clear();
        }
    }
}

/// Statistics panel
pub struct StatsPanel;

impl StatsPanel {
    pub fn new() -> Self {
        Self
    }
    
    pub fn show(&self, ui: &mut egui::Ui, state: &GuiState) {
        ui.heading("📊 Statistics");
        ui.separator();
        
        ui.label(format!("Active Sensors: {}", state.stats.active_sensors));
        ui.label(format!("Readings/sec: {:.0}", state.stats.readings_per_sec));
        ui.label(format!("Total Detections: {}", state.stats.detections_total));
        
        ui.separator();
        
        ui.label(format!("CPU: {:.1}%", state.stats.cpu_usage));
        
        // CPU bar
        let cpu_ratio = state.stats.cpu_usage / 100.0;
        ui.add(egui::ProgressBar::new(cpu_ratio).show_percentage());
        
        ui.label(format!("Memory: {:.0} MB", state.stats.memory_mb));
        
        ui.separator();
        
        let hours = state.stats.uptime_secs / 3600;
        let mins = (state.stats.uptime_secs % 3600) / 60;
        let secs = state.stats.uptime_secs % 60;
        ui.label(format!("Uptime: {:02}:{:02}:{:02}", hours, mins, secs));
    }
}

/// Colormap enum
#[derive(Clone, Copy)]
pub enum Colormap {
    Inferno,
    Viridis,
    Plasma,
    Turbo,
    Grayscale,
}

impl Colormap {
    pub fn to_color(&self, t: f32) -> egui::Color32 {
        let t = t.clamp(0.0, 1.0);
        
        match self {
            Colormap::Inferno => {
                // Inferno colormap approximation
                let r = (255.0 * (-4.545 * t.powi(3) + 5.014 * t.powi(2) + 0.491 * t).clamp(0.0, 1.0)) as u8;
                let g = (255.0 * (2.068 * t.powi(3) - 2.861 * t.powi(2) + 1.338 * t).clamp(0.0, 1.0)) as u8;
                let b = (255.0 * (-2.213 * t.powi(3) + 3.009 * t.powi(2) + 0.1 * t + 0.163).clamp(0.0, 1.0)) as u8;
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Viridis => {
                let r = (255.0 * (0.267 + 0.329 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                let g = (255.0 * (0.004 + 0.873 * t - 0.378 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                let b = (255.0 * (0.329 + 0.311 * t - 0.640 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Turbo => {
                let r = (255.0 * (0.18995 + 2.31 * t - 1.5 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                let g = (255.0 * (0.07176 + 2.89 * t - 2.0 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                let b = (255.0 * (0.23217 + 1.26 * t - 1.5 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Plasma => {
                let r = (255.0 * (0.05 + 0.91 * t).clamp(0.0, 1.0)) as u8;
                let g = (255.0 * (0.02 + 0.53 * t - 0.55 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                let b = (255.0 * (0.53 - 0.03 * t - 0.5 * t.powi(2)).clamp(0.0, 1.0)) as u8;
                egui::Color32::from_rgb(r, g, b)
            }
            Colormap::Grayscale => {
                let v = (255.0 * t) as u8;
                egui::Color32::from_rgb(v, v, v)
            }
        }
    }
}

fn get_sensor_color(sensor_id: &str) -> egui::Color32 {
    match sensor_id {
        s if s.contains("EMF") => egui::Color32::from_rgb(100, 200, 255),
        s if s.contains("Thermal") => egui::Color32::from_rgb(255, 100, 100),
        s if s.contains("Audio") => egui::Color32::from_rgb(100, 255, 100),
        s if s.contains("Seismic") => egui::Color32::from_rgb(255, 200, 100),
        _ => egui::Color32::from_rgb(200, 200, 200),
    }
}
