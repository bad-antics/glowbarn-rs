//! Quantum and random number sensors

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;
use rand::prelude::*;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Quantum Random Number Generator
pub struct QRNGSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    source_type: QRNGSourceType,
}

#[derive(Clone, Copy)]
pub enum QRNGSourceType {
    PhotonArrival,    // Photon timing
    BeamSplitter,     // Quantum beam splitter
    Vacuum,           // Vacuum fluctuations
    RadioactiveDecay, // True random from decay
}

impl QRNGSensor {
    pub fn new(id: &str, source_type: QRNGSourceType) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1000.0,  // 1000 random numbers per second
            sequence: 0,
            source_type,
        }
    }
}

#[async_trait]
impl Sensor for QRNGSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::QRNG }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.0,  // Perfect randomness has no noise floor
            timestamp: Utc::now(),
            temperature: None,
            notes: match self.source_type {
                QRNGSourceType::PhotonArrival => "QRNG: Photon arrival time".to_string(),
                QRNGSourceType::BeamSplitter => "QRNG: Quantum beam splitter".to_string(),
                QRNGSourceType::Vacuum => "QRNG: Vacuum fluctuations".to_string(),
                QRNGSourceType::RadioactiveDecay => "QRNG: Radioactive decay".to_string(),
            },
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Thermal noise (Johnson-Nyquist) random source
pub struct ThermalNoiseSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    resistance: f64,  // Ohms
    temperature: f64, // Kelvin
}

impl ThermalNoiseSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10000.0,
            sequence: 0,
            resistance: 10000.0,  // 10k ohm
            temperature: 300.0,   // Room temperature
        }
    }
    
    /// Calculate theoretical noise voltage (RMS)
    /// V = sqrt(4 * k_B * T * R * bandwidth)
    pub fn theoretical_noise_vrms(&self, bandwidth: f64) -> f64 {
        let kb = 1.380649e-23;  // Boltzmann constant
        (4.0 * kb * self.temperature * self.resistance * bandwidth).sqrt()
    }
}

#[async_trait]
impl Sensor for ThermalNoiseSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ThermalNoise }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        let noise_v = self.theoretical_noise_vrms(self.sample_rate / 2.0);
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0 / noise_v],  // Normalize to unit variance
            noise_floor: noise_v,
            timestamp: Utc::now(),
            temperature: Some(self.temperature - 273.15),  // Convert to Celsius
            notes: format!("Thermal noise: R={} Ω, T={} K", self.resistance, self.temperature),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "resistance": self.resistance,
            "temperature": self.temperature
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(r) = config.get("resistance").and_then(|v| v.as_f64()) {
            self.resistance = r;
        }
        if let Some(t) = config.get("temperature").and_then(|v| v.as_f64()) {
            self.temperature = t;
        }
        Ok(())
    }
}

/// Shot noise random source
pub struct ShotNoiseSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    current: f64,  // Amps
}

impl ShotNoiseSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10000.0,
            sequence: 0,
            current: 1e-6,  // 1 µA
        }
    }
    
    /// Shot noise current (RMS)
    /// I_shot = sqrt(2 * q * I * bandwidth)
    pub fn theoretical_noise_arms(&self, bandwidth: f64) -> f64 {
        let q = 1.602176634e-19;  // Elementary charge
        (2.0 * q * self.current * bandwidth).sqrt()
    }
}

#[async_trait]
impl Sensor for ShotNoiseSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ShotNoise }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        let noise_a = self.theoretical_noise_arms(self.sample_rate / 2.0);
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0 / noise_a],
            noise_floor: noise_a,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Shot noise: I={} µA", self.current * 1e6),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"current": self.current}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(c) = config.get("current").and_then(|v| v.as_f64()) {
            self.current = c;
        }
        Ok(())
    }
}

/// Zener diode avalanche noise source
pub struct ZenerNoiseSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    breakdown_voltage: f64,
}

impl ZenerNoiseSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100000.0,  // High bandwidth
            sequence: 0,
            breakdown_voltage: 5.1,  // Typical 5.1V Zener
        }
    }
}

#[async_trait]
impl Sensor for ZenerNoiseSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::ZenerDiode }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.001,  // ~1mV noise
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Zener avalanche noise: Vz={} V", self.breakdown_voltage),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"breakdown_voltage": self.breakdown_voltage}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(v) = config.get("breakdown_voltage").and_then(|v| v.as_f64()) {
            self.breakdown_voltage = v;
        }
        Ok(())
    }
}
