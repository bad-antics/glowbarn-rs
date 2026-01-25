//! Main application window

use eframe::egui;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::Utc;

use crate::config::Config;
use crate::sensors::{SensorManager, SensorReading, SensorType};
use crate::detection::{Detection, DetectionType, Severity};
use super::{GuiState, SystemStats, ThermalData, SpectrumData};
use super::panels::*;
use super::widgets::*;
use super::theme::*;

/// Main GlowBarn application
pub struct GlowBarnApp {
    config: Config,
    state: GuiState,
    
    // Panels
    sensor_panel: SensorPanel,
    waveform_panel: WaveformPanel,
    thermal_panel: ThermalPanel,
    spectrum_panel: SpectrumPanel,
    detection_panel: DetectionPanel,
    stats_panel: StatsPanel,
    
    // Demo data generation
    demo_mode: bool,
    frame_count: u64,
    
    // Frame timing
    last_update: std::time::Instant,
}

impl GlowBarnApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        let demo_mode = config.demo_mode;
        
        Self {
            config,
            state: GuiState::default(),
            sensor_panel: SensorPanel::new(),
            waveform_panel: WaveformPanel::new(),
            thermal_panel: ThermalPanel::new(),
            spectrum_panel: SpectrumPanel::new(),
            detection_panel: DetectionPanel::new(),
            stats_panel: StatsPanel::new(),
            demo_mode,
            frame_count: 0,
            last_update: std::time::Instant::now(),
        }
    }
    
    fn update_demo_data(&mut self) {
        let t = self.frame_count as f64 * 0.05;
        
        // Generate demo waveform data
        for sensor_id in ["EMF-001", "Thermal-001", "Audio-001", "Seismic-001"] {
            let waveform = self.state.waveforms
                .entry(sensor_id.to_string())
                .or_insert_with(Vec::new);
            
            // Generate different patterns for each sensor
            let value = match sensor_id {
                "EMF-001" => (t * 0.3).sin() * 50.0 + 100.0 + (t * 2.1).sin() * 10.0,
                "Thermal-001" => 22.0 + (t * 0.1).sin() * 2.0 + rand_f64() * 0.5,
                "Audio-001" => (t * 5.0).sin() * 0.5 + rand_f64() * 0.2,
                "Seismic-001" => (t * 0.5).sin() * 0.01 + rand_f64() * 0.002,
                _ => 0.0,
            };
            
            waveform.push(value);
            
            // Keep last 500 samples
            if waveform.len() > 500 {
                waveform.drain(0..waveform.len() - 500);
            }
        }
        
        // Generate demo thermal data
        if self.frame_count % 10 == 0 {
            let mut thermal = vec![0.0f32; 24 * 32];
            for y in 0..24 {
                for x in 0..32 {
                    let base = 22.0 + (x as f32 - 16.0).abs() * 0.1 + (y as f32 - 12.0).abs() * 0.1;
                    let noise = rand_f64() as f32 * 0.5;
                    
                    // Add a "hot spot" that moves
                    let spot_x = 16.0 + (t * 0.2).sin() as f32 * 8.0;
                    let spot_y = 12.0 + (t * 0.3).cos() as f32 * 6.0;
                    let dist = ((x as f32 - spot_x).powi(2) + (y as f32 - spot_y).powi(2)).sqrt();
                    let hot_spot = 5.0 * (-dist / 3.0).exp();
                    
                    thermal[y * 32 + x] = base + noise + hot_spot;
                }
            }
            
            self.state.thermal_data = Some(ThermalData {
                width: 32,
                height: 24,
                data: thermal.clone(),
                min_temp: thermal.iter().fold(f32::MAX, |a, &b| a.min(b)),
                max_temp: thermal.iter().fold(f32::MIN, |a, &b| a.max(b)),
                timestamp: Utc::now(),
            });
        }
        
        // Generate demo spectrum data
        if self.frame_count % 5 == 0 {
            let mut frequencies = Vec::new();
            let mut magnitudes = Vec::new();
            let mut max_mag = 0.0f32;
            let mut peak_freq = 0.0f32;
            
            for i in 0..256 {
                let freq = i as f32 * 100.0;  // Up to 25.6 kHz
                frequencies.push(freq);
                
                // Multiple peaks
                let mag = 
                    10.0 * (-(freq - 1000.0).abs() / 200.0).exp() +  // 1 kHz peak
                    5.0 * (-(freq - 5000.0).abs() / 500.0).exp() +   // 5 kHz peak
                    2.0 * rand_f64() as f32;  // Noise floor
                
                magnitudes.push(mag);
                
                if mag > max_mag {
                    max_mag = mag;
                    peak_freq = freq;
                }
            }
            
            self.state.spectrum_data = Some(SpectrumData {
                frequencies,
                magnitudes,
                peak_freq,
                timestamp: Utc::now(),
            });
        }
        
        // Generate occasional detections
        if self.frame_count % 200 == 0 && rand_f64() > 0.5 {
            let detection = Detection {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now(),
                detection_type: match (rand_f64() * 5.0) as u32 {
                    0 => DetectionType::EMFSpike,
                    1 => DetectionType::ThermalAnomaly,
                    2 => DetectionType::InfrasoundEvent,
                    3 => DetectionType::CorrelatedAnomaly,
                    _ => DetectionType::EntropyAnomaly,
                },
                confidence: 0.5 + rand_f64() * 0.5,
                severity: match (rand_f64() * 4.0) as u32 {
                    0 => Severity::Low,
                    1 => Severity::Medium,
                    2 => Severity::High,
                    _ => Severity::Critical,
                },
                sensors: vec![],
                entropy_deviation: rand_f64() * 0.3,
                anomaly_count: (rand_f64() * 5.0) as usize,
                correlation_score: rand_f64() * 0.8,
                classification: None,
                location: None,
                data_window_start: Utc::now(),
                data_window_end: Utc::now(),
            };
            
            self.state.detections.push(detection);
            
            // Keep last 100 detections
            if self.state.detections.len() > 100 {
                self.state.detections.drain(0..self.state.detections.len() - 100);
            }
        }
        
        // Update stats
        let elapsed = self.last_update.elapsed().as_secs_f64();
        self.state.stats = SystemStats {
            readings_per_sec: 100.0,
            detections_total: self.state.detections.len(),
            cpu_usage: 15.0 + rand_f64() as f32 * 10.0,
            memory_mb: 128.0 + rand_f64() * 50.0,
            uptime_secs: (self.frame_count / 60) as u64,
            active_sensors: 14,
        };
    }
}

impl eframe::App for GlowBarnApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        
        // Update demo data
        if self.demo_mode {
            self.update_demo_data();
        }
        
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Session").clicked() {
                        ui.close_menu();
                    }
                    if ui.button("Export Data...").clicked() {
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.state.show_settings, "Settings");
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.state.show_about = true;
                        ui.close_menu();
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Recording indicator
                    if self.state.recording {
                        ui.colored_label(egui::Color32::RED, "⏺ RECORDING");
                    }
                    
                    // Demo mode indicator
                    if self.demo_mode {
                        ui.label("🎭 Demo Mode");
                    }
                    
                    // FPS
                    if self.config.gui.show_fps {
                        ui.label(format!("{:.0} FPS", 1.0 / ctx.input(|i| i.predicted_dt)));
                    }
                });
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                status_indicator(ui, true, "System");
                ui.separator();
                ui.label(format!("Sensors: {}", self.state.stats.active_sensors));
                ui.separator();
                ui.label(format!("Readings/s: {:.0}", self.state.stats.readings_per_sec));
                ui.separator();
                ui.label(format!("Detections: {}", self.state.stats.detections_total));
                ui.separator();
                ui.label(format!("CPU: {:.1}%", self.state.stats.cpu_usage));
                ui.separator();
                ui.label(format!("Mem: {:.0} MB", self.state.stats.memory_mb));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(chrono::Local::now().format("%H:%M:%S").to_string());
                });
            });
        });
        
        // Left panel - Sensor list
        egui::SidePanel::left("sensor_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                self.sensor_panel.show(ui, &mut self.state);
            });
        
        // Right panel - Detections
        egui::SidePanel::right("detection_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                self.detection_panel.show(ui, &mut self.state);
            });
        
        // Central panel with visualizations
        egui::CentralPanel::default().show(ctx, |ui| {
            // Split into top and bottom
            let available_height = ui.available_height();
            
            // Top row - Waveforms and Thermal
            egui::TopBottomPanel::top("viz_top")
                .resizable(true)
                .default_height(available_height * 0.5)
                .show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Waveforms
                        ui.group(|ui| {
                            ui.set_min_width(ui.available_width() * 0.6);
                            self.waveform_panel.show(ui, &self.state);
                        });
                        
                        // Thermal
                        ui.group(|ui| {
                            self.thermal_panel.show(ui, &self.state);
                        });
                    });
                });
            
            // Bottom row - Spectrum and Stats
            ui.horizontal(|ui| {
                // Spectrum
                ui.group(|ui| {
                    ui.set_min_width(ui.available_width() * 0.7);
                    self.spectrum_panel.show(ui, &self.state);
                });
                
                // Stats
                ui.group(|ui| {
                    self.stats_panel.show(ui, &self.state);
                });
            });
        });
        
        // Settings window
        if self.state.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.state.show_settings)
                .show(ctx, |ui| {
                    ui.heading("General");
                    ui.checkbox(&mut self.demo_mode, "Demo Mode");
                    ui.checkbox(&mut self.state.alerts_enabled, "Enable Alerts");
                    
                    ui.separator();
                    ui.heading("Display");
                    // Theme selection, etc.
                });
        }
        
        // About window
        if self.state.show_about {
            egui::Window::new("About GlowBarn")
                .open(&mut self.state.show_about)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("🌟 GlowBarn");
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.label("High-Performance Paranormal Detection Suite");
                        ui.separator();
                        ui.label("Built with Rust 🦀");
                        ui.label("50+ sensor types • Advanced entropy analysis");
                        ui.label("Military-grade encryption • GPU acceleration");
                    });
                });
        }
        
        // Request continuous repainting for real-time updates
        ctx.request_repaint();
        
        self.last_update = std::time::Instant::now();
    }
}

// Simple random number generator for demo
fn rand_f64() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos as f64 / u32::MAX as f64)
}
