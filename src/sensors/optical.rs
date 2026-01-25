// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Optical sensors - spectrometers, light meters, LiDAR, laser grids

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Light meter / Lux sensor
pub struct LightMeterSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl LightMeterSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for LightMeterSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::LightMeter }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.1,  // lux
            timestamp: Utc::now(),
            temperature: None,
            notes: "Light meter calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// UV sensor (A/B/C bands)
pub struct UVSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl UVSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for UVSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::UVSensor }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0, 0.0, 0.0],  // UV-A, UV-B, UV-C
            scale: vec![1.0, 1.0, 1.0],
            noise_floor: 0.001,  // mW/cm²
            timestamp: Utc::now(),
            temperature: None,
            notes: "UV sensor calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Visible light spectrometer
pub struct SpectrometerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    num_channels: usize,
    wavelength_range: (f64, f64),  // nm
}

impl SpectrometerSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,
            sequence: 0,
            num_channels: 512,
            wavelength_range: (380.0, 780.0),
        }
    }
}

#[async_trait]
impl Sensor for SpectrometerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Spectrometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; self.num_channels],
            scale: vec![1.0; self.num_channels],
            noise_floor: 0.01,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Spectrometer calibration, {} channels, {}-{} nm",
                self.num_channels, self.wavelength_range.0, self.wavelength_range.1),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "num_channels": self.num_channels,
            "wavelength_range": self.wavelength_range
        })
    }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// LiDAR distance/mapping sensor
pub struct LiDARSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    max_range: f64,  // meters
    angular_resolution: f64,  // degrees
}

impl LiDARSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,  // Scans per second
            sequence: 0,
            max_range: 12.0,
            angular_resolution: 0.5,
        }
    }
}

#[async_trait]
impl Sensor for LiDARSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::LiDAR }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.02,  // 2cm accuracy
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("LiDAR calibration, max range: {}m", self.max_range),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "max_range": self.max_range,
            "angular_resolution": self.angular_resolution
        })
    }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Laser grid interruption detector
pub struct LaserGridSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    num_beams: usize,
    grid_dimensions: (usize, usize),  // rows, cols
}

impl LaserGridSensor {
    pub fn new(id: &str, rows: usize, cols: usize) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 60.0,
            sequence: 0,
            num_beams: rows + cols,
            grid_dimensions: (rows, cols),
        }
    }
}

#[async_trait]
impl Sensor for LaserGridSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::LaserGrid }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; self.num_beams],
            scale: vec![1.0; self.num_beams],
            noise_floor: 0.01,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Laser grid {}x{}", self.grid_dimensions.0, self.grid_dimensions.1),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "num_beams": self.num_beams,
            "grid_dimensions": self.grid_dimensions
        })
    }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
