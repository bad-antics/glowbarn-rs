// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Theme configuration

use eframe::egui;
use crate::config::Theme;

/// Apply theme to egui context
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    match theme {
        Theme::Dark => apply_dark_theme(ctx),
        Theme::Light => apply_light_theme(ctx),
        Theme::System => {
            // Could detect system theme here
            apply_dark_theme(ctx);
        }
    }
}

fn apply_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Use dark visuals
    style.visuals = egui::Visuals::dark();
    
    // Customize colors
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(25, 25, 30);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 35, 40);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(50, 50, 60);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 70);
    
    // Selection color
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(50, 100, 150);
    
    // Panel background
    style.visuals.panel_fill = egui::Color32::from_rgb(20, 20, 25);
    style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 35);
    
    // Extreme background (darkest areas)
    style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 15, 18);
    
    // Faint (subtle lines and borders)
    style.visuals.faint_bg_color = egui::Color32::from_rgb(35, 35, 40);
    
    // Accent color for highlights
    style.visuals.hyperlink_color = egui::Color32::from_rgb(100, 200, 255);
    
    // Window shadow
    style.visuals.window_shadow = egui::epaint::Shadow::small_dark();
    
    // Rounded corners
    style.visuals.window_rounding = egui::Rounding::same(6.0);
    style.visuals.menu_rounding = egui::Rounding::same(4.0);
    
    // Button rounding
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(4.0);
    
    // Spacing
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    
    ctx.set_style(style);
}

fn apply_light_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    style.visuals = egui::Visuals::light();
    
    // Customize for light mode
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(240, 240, 245);
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(230, 230, 235);
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(220, 220, 230);
    style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(200, 200, 220);
    
    style.visuals.panel_fill = egui::Color32::from_rgb(248, 248, 250);
    style.visuals.window_fill = egui::Color32::from_rgb(255, 255, 255);
    
    style.visuals.selection.bg_fill = egui::Color32::from_rgb(100, 150, 200);
    
    // Rounded corners
    style.visuals.window_rounding = egui::Rounding::same(6.0);
    style.visuals.menu_rounding = egui::Rounding::same(4.0);
    
    ctx.set_style(style);
}

/// GlowBarn specific color palette
pub struct GlowBarnColors;

impl GlowBarnColors {
    pub const PRIMARY: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
    pub const SECONDARY: egui::Color32 = egui::Color32::from_rgb(150, 100, 255);
    pub const SUCCESS: egui::Color32 = egui::Color32::from_rgb(100, 255, 150);
    pub const WARNING: egui::Color32 = egui::Color32::from_rgb(255, 200, 100);
    pub const DANGER: egui::Color32 = egui::Color32::from_rgb(255, 100, 100);
    pub const INFO: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
    
    // Sensor type colors
    pub const EMF: egui::Color32 = egui::Color32::from_rgb(100, 200, 255);
    pub const THERMAL: egui::Color32 = egui::Color32::from_rgb(255, 100, 100);
    pub const AUDIO: egui::Color32 = egui::Color32::from_rgb(100, 255, 150);
    pub const SEISMIC: egui::Color32 = egui::Color32::from_rgb(255, 200, 100);
    pub const RADIATION: egui::Color32 = egui::Color32::from_rgb(255, 255, 100);
    pub const RF: egui::Color32 = egui::Color32::from_rgb(200, 100, 255);
    pub const OPTICAL: egui::Color32 = egui::Color32::from_rgb(255, 150, 200);
    pub const QUANTUM: egui::Color32 = egui::Color32::from_rgb(100, 255, 255);
    
    // Severity colors  
    pub const SEVERITY_LOW: egui::Color32 = egui::Color32::from_rgb(100, 200, 100);
    pub const SEVERITY_MEDIUM: egui::Color32 = egui::Color32::from_rgb(200, 200, 100);
    pub const SEVERITY_HIGH: egui::Color32 = egui::Color32::from_rgb(255, 150, 50);
    pub const SEVERITY_CRITICAL: egui::Color32 = egui::Color32::from_rgb(255, 50, 50);
}

/// Get color for sensor type
pub fn sensor_type_color(sensor_type: &crate::sensors::SensorType) -> egui::Color32 {
    use crate::sensors::SensorType;
    
    match sensor_type {
        SensorType::EMFProbe | SensorType::FluxGate | SensorType::TriField => GlowBarnColors::EMF,
        SensorType::ThermalArray | SensorType::ThermalImager => GlowBarnColors::THERMAL,
        SensorType::Infrasound | SensorType::Ultrasonic | SensorType::FullSpectrum | 
        SensorType::ParabolicMic | SensorType::MicArray => GlowBarnColors::AUDIO,
        SensorType::Accelerometer | SensorType::Geophone | SensorType::Accelerometer => GlowBarnColors::SEISMIC,
        SensorType::GeigerCounter | SensorType::Scintillator | SensorType::NeutronDetector => GlowBarnColors::RADIATION,
        SensorType::SDRReceiver | SensorType::SpectrumAnalyzer | SensorType::WiFiScanner => GlowBarnColors::RF,
        SensorType::LightMeter | SensorType::UVSensor | SensorType::Spectrometer | 
        SensorType::LiDAR | SensorType::LaserGrid => GlowBarnColors::OPTICAL,
        SensorType::QRNG | SensorType::ThermalNoise | SensorType::ShotNoise | 
        SensorType::ZenerDiode => GlowBarnColors::QUANTUM,
        _ => egui::Color32::GRAY,
    }
}

/// Get color for severity
pub fn severity_color(severity: &crate::detection::Severity) -> egui::Color32 {
    match severity {
        crate::detection::Severity::Low => GlowBarnColors::SEVERITY_LOW,
        crate::detection::Severity::Medium => GlowBarnColors::SEVERITY_MEDIUM,
        crate::detection::Severity::High => GlowBarnColors::SEVERITY_HIGH,
        crate::detection::Severity::Critical => GlowBarnColors::SEVERITY_CRITICAL,
    }
}
