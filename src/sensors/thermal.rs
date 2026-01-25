// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Thermal sensors - infrared imaging and temperature measurement

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// MLX90640 Far Infrared Thermal Sensor Array
pub struct MLX90640Sensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    #[cfg(feature = "serial")]
    port: Option<Box<dyn serialport::SerialPort>>,
}

impl MLX90640Sensor {
    pub fn new(id: &str, port_name: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 4.0,  // Max 64Hz
            sequence: 0,
            #[cfg(feature = "serial")]
            port: None,
        }
    }
}

#[async_trait]
impl Sensor for MLX90640Sensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ThermalArray }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> {
        self.status = SensorStatus::Connected;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.status = SensorStatus::Disconnected;
        Ok(())
    }
    
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; 768],
            scale: vec![1.0; 768],
            noise_floor: 0.5,
            timestamp: Utc::now(),
            temperature: Some(25.0),
            notes: "MLX90640 factory calibration".to_string(),
            signature: vec![],
        })
    }
    
    async fn read(&mut self) -> Result<SensorReading> {
        self.sequence += 1;
        bail!("Hardware not connected - use simulator")
    }
    
    fn sample_rate(&self) -> f64 { self.sample_rate }
    
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> {
        self.sample_rate = rate.min(64.0);
        Ok(())
    }
    
    fn config(&self) -> serde_json::Value {
        serde_json::json!({ "refresh_rate": self.sample_rate })
    }
    
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(rate) = config.get("refresh_rate").and_then(|v| v.as_f64()) {
            self.sample_rate = rate.min(64.0);
        }
        Ok(())
    }
}

/// AMG8833 Grid-EYE Infrared Array Sensor
pub struct AMG8833Sensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl AMG8833Sensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,  // Max 10Hz
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for AMG8833Sensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ThermalArray }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> {
        self.status = SensorStatus::Connected;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.status = SensorStatus::Disconnected;
        Ok(())
    }
    
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; 64],
            scale: vec![1.0; 64],
            noise_floor: 0.25,
            timestamp: Utc::now(),
            temperature: Some(25.0),
            notes: "AMG8833 calibration".to_string(),
            signature: vec![],
        })
    }
    
    async fn read(&mut self) -> Result<SensorReading> {
        self.sequence += 1;
        bail!("Hardware not connected - use simulator")
    }
    
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> {
        self.sample_rate = rate.min(10.0);
        Ok(())
    }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
