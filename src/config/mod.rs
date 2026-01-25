// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Configuration module

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

use crate::security::SecurityConfig;
use crate::streaming::StreamingConfig;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Application name
    pub app_name: String,
    
    /// Application version
    pub version: String,
    
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Log level
    pub log_level: String,
    
    /// Enable demo mode (simulated sensors)
    pub demo_mode: bool,
    
    /// Sensor configuration
    pub sensors: SensorConfig,
    
    /// Analysis configuration
    pub analysis: AnalysisConfig,
    
    /// Detection configuration  
    pub detection: DetectionConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Streaming configuration
    pub streaming: StreamingConfig,
    
    /// GUI configuration
    pub gui: GuiConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_name: "GlowBarn".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            demo_mode: true,
            sensors: SensorConfig::default(),
            analysis: AnalysisConfig::default(),
            detection: DetectionConfig::default(),
            security: SecurityConfig::default(),
            streaming: StreamingConfig::default(),
            gui: GuiConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        info!("Loaded configuration from {:?}", path);
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        info!("Saved configuration to {:?}", path);
        Ok(())
    }
    
    /// Load or create default configuration
    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let config = Self::default();
            
            // Create parent directories
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            config.save(path)?;
            Ok(config)
        }
    }
    
    /// Get configuration directory
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .map(|d| d.join("glowbarn"))
            .unwrap_or_else(|| PathBuf::from("./config"))
    }
    
    /// Get default configuration path
    pub fn default_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }
}

/// Sensor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    /// Sample rate in Hz
    pub sample_rate: f64,
    
    /// Buffer size for readings
    pub buffer_size: usize,
    
    /// Calibration interval in seconds
    pub calibration_interval_secs: u64,
    
    /// Enable auto-discovery of hardware sensors
    pub auto_discover: bool,
    
    /// Serial port for hardware sensors
    pub serial_port: Option<String>,
    
    /// I2C bus number
    pub i2c_bus: Option<u8>,
    
    /// SPI device
    pub spi_device: Option<String>,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 100.0,
            buffer_size: 10000,
            calibration_interval_secs: 3600,
            auto_discover: true,
            serial_port: None,
            i2c_bus: Some(1),
            spi_device: None,
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Window size for entropy analysis
    pub entropy_window: usize,
    
    /// Anomaly detection threshold
    pub anomaly_threshold: f64,
    
    /// FFT size
    pub fft_size: usize,
    
    /// Enable GPU acceleration
    pub gpu_enabled: bool,
    
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Enable multi-scale entropy
    pub multiscale_entropy: bool,
    
    /// Number of entropy scales
    pub entropy_scales: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            entropy_window: 1000,
            anomaly_threshold: 0.7,
            fft_size: 2048,
            gpu_enabled: false,
            worker_threads: 4,
            multiscale_entropy: true,
            entropy_scales: 10,
        }
    }
}

/// Detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Minimum confidence for detection
    pub min_confidence: f64,
    
    /// Enable multi-sensor fusion
    pub fusion_enabled: bool,
    
    /// Fusion method
    pub fusion_method: FusionMethod,
    
    /// Correlation window in milliseconds
    pub correlation_window_ms: u64,
    
    /// Minimum sensors for correlated event
    pub min_correlated_sensors: usize,
    
    /// Enable classification
    pub classification_enabled: bool,
    
    /// Alert severity threshold
    pub alert_threshold: Severity,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            fusion_enabled: true,
            fusion_method: FusionMethod::DempsterShafer,
            correlation_window_ms: 2000,
            min_correlated_sensors: 2,
            classification_enabled: true,
            alert_threshold: Severity::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FusionMethod {
    Bayesian,
    DempsterShafer,
    WeightedAverage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// GUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiConfig {
    /// Window width
    pub width: u32,
    
    /// Window height
    pub height: u32,
    
    /// Enable VSync
    pub vsync: bool,
    
    /// Theme
    pub theme: Theme,
    
    /// Font size
    pub font_size: f32,
    
    /// Show FPS counter
    pub show_fps: bool,
    
    /// Waveform history length
    pub waveform_history: usize,
    
    /// Thermal colormap
    pub thermal_colormap: Colormap,
    
    /// Alert sound enabled
    pub alert_sound: bool,
}

impl Default for GuiConfig {
    fn default() -> Self {
        Self {
            width: 1400,
            height: 900,
            vsync: true,
            theme: Theme::Dark,
            font_size: 14.0,
            show_fps: false,
            waveform_history: 500,
            thermal_colormap: Colormap::Inferno,
            alert_sound: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Colormap {
    Inferno,
    Viridis,
    Plasma,
    Magma,
    Turbo,
    Grayscale,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Enable database storage
    pub enabled: bool,
    
    /// Database path
    pub path: PathBuf,
    
    /// Maximum database size in MB
    pub max_size_mb: u64,
    
    /// Retention period in days
    pub retention_days: u32,
    
    /// Flush interval in seconds
    pub flush_interval_secs: u64,
    
    /// Enable compression
    pub compression: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            path: PathBuf::from("./data/glowbarn.db"),
            max_size_mb: 1024,
            retention_days: 30,
            flush_interval_secs: 10,
            compression: true,
        }
    }
}
