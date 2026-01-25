// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Environmental sensors - pressure, humidity, air quality

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// BMP280/BME280 Barometric pressure sensor
pub struct BarometerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    altitude_correction: f64,
}

impl BarometerSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
            altitude_correction: 0.0,
        }
    }
}

#[async_trait]
impl Sensor for BarometerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Barometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![self.altitude_correction],
            scale: vec![1.0],
            noise_floor: 0.12,  // ±0.12 hPa
            timestamp: Utc::now(),
            temperature: None,
            notes: "BMP280 barometer calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate.min(157.0); Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"altitude_correction": self.altitude_correction}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(a) = config.get("altitude_correction").and_then(|v| v.as_f64()) {
            self.altitude_correction = a;
        }
        Ok(())
    }
}

/// Hygrometer - humidity sensor
pub struct HygrometerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl HygrometerSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for HygrometerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Hygrometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 2.0,  // ±2% RH
            timestamp: Utc::now(),
            temperature: None,
            notes: "Hygrometer calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// VOC (Volatile Organic Compounds) sensor
pub struct VOCSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl VOCSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for VOCSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::VOCSensor }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 5.0,  // ppb
            timestamp: Utc::now(),
            temperature: None,
            notes: "VOC sensor calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Particulate matter sensor (PM2.5/PM10)
pub struct ParticulateSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl ParticulateSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for ParticulateSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ParticulateSensor }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0, 0.0],  // PM2.5, PM10
            scale: vec![1.0, 1.0],
            noise_floor: 1.0,  // µg/m³
            timestamp: Utc::now(),
            temperature: None,
            notes: "Particulate sensor calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Anemometer - wind/air flow sensor
pub struct AnemometerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl AnemometerSensor {
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
impl Sensor for AnemometerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Anemometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.1,  // m/s
            timestamp: Utc::now(),
            temperature: None,
            notes: "Anemometer calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
