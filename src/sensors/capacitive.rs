// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Capacitive and electric field sensors

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Capacitive proximity sensor
pub struct CapacitiveSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    sensitivity: f64,
    threshold: f64,
}

impl CapacitiveSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,
            sequence: 0,
            sensitivity: 1.0,
            threshold: 0.1,
        }
    }
}

#[async_trait]
impl Sensor for CapacitiveSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::CapacitiveSensor }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![self.sensitivity],
            noise_floor: 0.01,
            timestamp: Utc::now(),
            temperature: None,
            notes: "Capacitive sensor calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "sensitivity": self.sensitivity,
            "threshold": self.threshold
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(s) = config.get("sensitivity").and_then(|v| v.as_f64()) {
            self.sensitivity = s;
        }
        if let Some(t) = config.get("threshold").and_then(|v| v.as_f64()) {
            self.threshold = t;
        }
        Ok(())
    }
}

/// Static electricity meter
pub struct StaticMeterSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    range: f64,  // V/m max
}

impl StaticMeterSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,
            sequence: 0,
            range: 20000.0,  // ±20 kV/m
        }
    }
}

#[async_trait]
impl Sensor for StaticMeterSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::StaticMeter }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 10.0,  // V/m
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Static meter calibration, range: ±{} kV/m", self.range / 1000.0),
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

/// Field mill for atmospheric electric field
pub struct FieldMillSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl FieldMillSensor {
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
impl Sensor for FieldMillSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::FieldMill }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 1.0,  // V/m
            timestamp: Utc::now(),
            temperature: None,
            notes: "Field mill calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// AC/DC current clamp sensor
pub struct CurrentClampSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    max_current: f64,  // Amps
}

impl CurrentClampSensor {
    pub fn new(id: &str, max_current: f64) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1000.0,  // For AC waveform capture
            sequence: 0,
            max_current,
        }
    }
}

#[async_trait]
impl Sensor for CurrentClampSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::CurrentClamp }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.01,  // 10mA
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Current clamp calibration, max: {} A", self.max_current),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"max_current": self.max_current}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(m) = config.get("max_current").and_then(|v| v.as_f64()) {
            self.max_current = m;
        }
        Ok(())
    }
}
