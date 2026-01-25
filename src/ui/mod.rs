// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! UI module - egui visual console

mod app;
mod panels;
mod widgets;
mod plots;
mod theme;

pub use app::*;
pub use panels::*;
pub use widgets::*;
pub use plots::*;
pub use theme::*;

use anyhow::Result;
use eframe::egui;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::Config;
use crate::sensors::SensorReading;
use crate::detection::Detection;
use crate::core::EventBus;

/// GUI state
pub struct GuiState {
    /// Current sensor readings
    pub readings: Vec<SensorReading>,
    
    /// Recent detections
    pub detections: Vec<Detection>,
    
    /// Waveform history
    pub waveforms: std::collections::HashMap<String, Vec<f64>>,
    
    /// Thermal grid data
    pub thermal_data: Option<ThermalData>,
    
    /// Spectrum data
    pub spectrum_data: Option<SpectrumData>,
    
    /// System stats
    pub stats: SystemStats,
    
    /// Selected sensor
    pub selected_sensor: Option<String>,
    
    /// Show settings
    pub show_settings: bool,
    
    /// Show about
    pub show_about: bool,
    
    /// Recording state
    pub recording: bool,
    
    /// Alert enabled
    pub alerts_enabled: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            readings: Vec::new(),
            detections: Vec::new(),
            waveforms: std::collections::HashMap::new(),
            thermal_data: None,
            spectrum_data: None,
            stats: SystemStats::default(),
            selected_sensor: None,
            show_settings: false,
            show_about: false,
            recording: false,
            alerts_enabled: true,
        }
    }
}

/// Thermal image data
#[derive(Debug, Clone)]
pub struct ThermalData {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f32>,
    pub min_temp: f32,
    pub max_temp: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Spectrum/FFT data
#[derive(Debug, Clone)]
pub struct SpectrumData {
    pub frequencies: Vec<f32>,
    pub magnitudes: Vec<f32>,
    pub peak_freq: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// System statistics
#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub readings_per_sec: f64,
    pub detections_total: usize,
    pub cpu_usage: f32,
    pub memory_mb: f64,
    pub uptime_secs: u64,
    pub active_sensors: usize,
}

/// Launch GUI application
pub fn run_gui(config: Config) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([config.gui.width as f32, config.gui.height as f32])
            .with_title("GlowBarn - Paranormal Detection Suite")
            .with_icon(load_icon()),
        vsync: config.gui.vsync,
        ..Default::default()
    };
    
    eframe::run_native(
        "GlowBarn",
        options,
        Box::new(|cc| {
            // Setup custom fonts
            setup_fonts(&cc.egui_ctx);
            
            // Apply theme
            apply_theme(&cc.egui_ctx, config.gui.theme);
            
            Box::new(GlowBarnApp::new(cc, config))
        }),
    ).map_err(|e| anyhow::anyhow!("Failed to run GUI: {}", e))
}

fn load_icon() -> egui::IconData {
    // Simple placeholder icon
    egui::IconData {
        rgba: vec![0u8, 255, 100, 255].repeat(32 * 32),
        width: 32,
        height: 32,
    }
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // You could load custom fonts here
    // fonts.font_data.insert("my_font".to_owned(), egui::FontData::from_static(include_bytes!("...")));
    
    ctx.set_fonts(fonts);
}
