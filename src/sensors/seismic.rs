//! Seismic and vibration sensors

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// ADXL345 Triple-Axis Accelerometer
pub struct ADXL345Sensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    range: i8,  // ±2g, ±4g, ±8g, ±16g
}

impl ADXL345Sensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,
            sequence: 0,
            range: 2,
        }
    }
}

#[async_trait]
impl Sensor for ADXL345Sensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Accelerometer }
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
            offset: vec![0.0, 0.0, 0.0],
            scale: vec![1.0, 1.0, 1.0],
            noise_floor: 0.004,  // 4 mg noise
            timestamp: Utc::now(),
            temperature: None,
            notes: "ADXL345 calibration".to_string(),
            signature: vec![],
        })
    }
    
    async fn read(&mut self) -> Result<SensorReading> {
        self.sequence += 1;
        bail!("Hardware not connected")
    }
    
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> {
        self.sample_rate = rate.min(3200.0);
        Ok(())
    }
    fn config(&self) -> serde_json::Value { serde_json::json!({"range": self.range}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(r) = config.get("range").and_then(|v| v.as_i64()) {
            self.range = match r {
                2 | 4 | 8 | 16 => r as i8,
                _ => 2,
            };
        }
        Ok(())
    }
}

/// MPU6050 6-Axis IMU (Accel + Gyro)
pub struct MPU6050Sensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl MPU6050Sensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,
            sequence: 0,
        }
    }
}

#[async_trait]
impl Sensor for MPU6050Sensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Accelerometer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; 6],
            scale: vec![1.0; 6],
            noise_floor: 0.004,
            timestamp: Utc::now(),
            temperature: None,
            notes: "MPU6050 calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate.min(1000.0); Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Geophone velocity sensor
pub struct GeophoneSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    sensitivity: f64,  // V/(m/s)
}

impl GeophoneSensor {
    pub fn new(id: &str, sensitivity: f64) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1000.0,
            sequence: 0,
            sensitivity,
        }
    }
}

#[async_trait]
impl Sensor for GeophoneSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Geophone }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0 / self.sensitivity],
            noise_floor: 1e-8,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Geophone sensitivity: {} V/(m/s)", self.sensitivity),
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
