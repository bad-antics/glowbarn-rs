// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! EMF sensors - electromagnetic field measurement

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Generic EMF probe sensor
pub struct EMFProbeSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    sensitivity: f64,  // mG/V
}

impl EMFProbeSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 50.0,
            sequence: 0,
            sensitivity: 10.0,
        }
    }
}

#[async_trait]
impl Sensor for EMFProbeSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::EMFProbe }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![self.sensitivity],
            noise_floor: 0.1,
            timestamp: Utc::now(),
            temperature: None,
            notes: "EMF probe calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"sensitivity": self.sensitivity}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(s) = config.get("sensitivity").and_then(|v| v.as_f64()) {
            self.sensitivity = s;
        }
        Ok(())
    }
}

/// HMC5883L 3-Axis Magnetometer
pub struct HMC5883LSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    gain: u8,
}

impl HMC5883LSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 75.0,  // Max 75Hz
            sequence: 0,
            gain: 1,
        }
    }
}

#[async_trait]
impl Sensor for HMC5883LSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::GaussMeter }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0, 0.0, 0.0],
            scale: vec![1.0, 1.0, 1.0],
            noise_floor: 0.002,  // 2 milliGauss
            timestamp: Utc::now(),
            temperature: None,
            notes: "HMC5883L calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate.min(75.0); Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"gain": self.gain}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(g) = config.get("gain").and_then(|v| v.as_u64()) {
            self.gain = g as u8;
        }
        Ok(())
    }
}

/// TriField meter (multi-axis EMF)
pub struct TriFieldSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    mode: TriFieldMode,
}

#[derive(Clone, Copy)]
pub enum TriFieldMode {
    Magnetic,
    Electric,
    Radio,
}

impl TriFieldSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,
            sequence: 0,
            mode: TriFieldMode::Magnetic,
        }
    }
}

#[async_trait]
impl Sensor for TriFieldSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::TriField }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.5,
            timestamp: Utc::now(),
            temperature: None,
            notes: "TriField calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
