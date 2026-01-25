// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Magnetic sensors - magnetometers, fluxgates, SQUID

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Fluxgate magnetometer - high precision vector magnetometer
pub struct FluxgateSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    range: f64,  // µT
}

impl FluxgateSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,
            sequence: 0,
            range: 100.0,  // ±100 µT
        }
    }
}

#[async_trait]
impl Sensor for FluxgateSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::FluxGate }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0, 0.0, 0.0],
            scale: vec![1.0, 1.0, 1.0],
            noise_floor: 0.001,  // 1 nT
            timestamp: Utc::now(),
            temperature: Some(25.0),
            notes: format!("Fluxgate calibration, range: ±{} µT", self.range),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"range": self.range}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(r) = config.get("range").and_then(|v| v.as_f64()) {
            self.range = r;
        }
        Ok(())
    }
}

/// SQUID magnetometer - ultra-sensitive superconducting sensor
pub struct SQUIDSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl SQUIDSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1000.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for SQUIDSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::SQUIDMagnetometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 1e-15,  // 1 fT (femtotesla) - extremely sensitive
            timestamp: Utc::now(),
            temperature: Some(4.2),  // Liquid helium temperature
            notes: "SQUID magnetometer calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Gradiometer - measures magnetic field gradient
pub struct GradiometerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    baseline: f64,  // Distance between sensors in meters
}

impl GradiometerSensor {
    pub fn new(id: &str, baseline: f64) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,
            sequence: 0,
            baseline,
        }
    }
}

#[async_trait]
impl Sensor for GradiometerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::FluxGate }  // Gradiometer variant
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0 / self.baseline],  // nT/m
            noise_floor: 0.01,  // nT/m
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Gradiometer calibration, baseline: {} m", self.baseline),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"baseline": self.baseline}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(b) = config.get("baseline").and_then(|v| v.as_f64()) {
            self.baseline = b;
        }
        Ok(())
    }
}
