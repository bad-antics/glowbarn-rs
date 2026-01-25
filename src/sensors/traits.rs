// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Sensor traits and common types

use std::time::Duration;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use nalgebra::DVector;
use anyhow::Result;

/// Sensor types supported by GlowBarn
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorType {
    // Thermal
    ThermalImager,      // FLIR, SEEK, MLX90640
    ThermalArray,       // AMG8833
    Thermistor,         // Point temperature
    Pyrometer,          // Non-contact IR
    
    // Seismic/Vibration
    Geophone,           // Velocity sensor
    Accelerometer,      // MEMS (ADXL, MPU, LIS)
    Seismograph,        // Raspberry Shake
    Piezoelectric,      // Contact vibration
    
    // Electromagnetic
    EMFProbe,           // Magnetic field
    TriField,           // Multi-axis EMF
    GaussMeter,         // Precision magnetic
    FluxGate,           // Vector magnetometer
    SQUIDMagnetometer,  // Ultra-sensitive
    
    // Audio
    Ultrasonic,         // >20kHz
    Infrasound,         // <20Hz
    FullSpectrum,       // 0.1Hz - 100kHz
    ParabolicMic,       // Directional
    ContactMic,         // Structure-borne
    MicArray,           // Beamforming array
    
    // Environmental
    Barometer,          // Pressure
    Hygrometer,         // Humidity
    Anemometer,         // Air flow
    IonCounter,         // Air ionization
    VOCSensor,          // Volatile organics
    ParticulateSensor,  // PM2.5/PM10
    
    // Radiation
    GeigerCounter,      // Alpha/Beta/Gamma
    Scintillator,       // Gamma spectroscopy
    NeutronDetector,    // Neutron flux
    DosimeterArray,     // Spatial radiation
    
    // Optical
    LightMeter,         // Luminosity
    UVSensor,           // UV A/B/C
    IRDetector,         // Passive IR
    Spectrometer,       // Full spectrum
    LiDAR,              // Distance/mapping
    LaserGrid,          // Interruption detection
    NightVision,        // Enhanced imaging
    
    // Radio Frequency
    SDRReceiver,        // Software-defined radio
    SpectrumAnalyzer,   // RF spectrum
    WiFiScanner,        // 2.4/5GHz
    EMIDetector,        // Interference
    
    // Capacitive/Electric
    CapacitiveSensor,   // Proximity
    StaticMeter,        // Electrostatic
    FieldMill,          // Atmospheric electric
    CurrentClamp,       // AC/DC current
    
    // Ionization
    IonChamber,         // Air ionization
    CoronaDetector,     // Electric discharge
    PlasmaProbe,        // Plasma detection
    
    // Quantum/Random
    QRNG,               // Quantum RNG
    ThermalNoise,       // Johnson noise
    ShotNoise,          // Electron shot noise
    ZenerDiode,         // Avalanche noise
    
    // Custom
    Custom(u32),        // User-defined
}

/// Sensor operational status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorStatus {
    Disconnected,
    Connecting,
    Connected,
    Calibrating,
    Active,
    Error,
    Maintenance,
}

/// Calibration data for a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub offset: Vec<f64>,
    pub scale: Vec<f64>,
    pub noise_floor: f64,
    pub timestamp: DateTime<Utc>,
    pub temperature: Option<f64>,
    pub notes: String,
    pub signature: Vec<u8>,  // Cryptographic signature
}

/// A single sensor reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    
    // Data
    pub data: Vec<f64>,
    pub dimensions: Vec<usize>,  // Shape for multi-dimensional data
    
    // Metadata
    pub unit: String,
    pub sample_rate: f64,
    pub quality: f32,  // 0-1 signal quality
    
    // Location (optional)
    pub position: Option<[f64; 3]>,  // x, y, z in meters
    pub orientation: Option<[f64; 3]>,  // roll, pitch, yaw in radians
}

impl SensorReading {
    pub fn new(sensor_id: &str, sensor_type: SensorType, data: Vec<f64>) -> Self {
        Self {
            sensor_id: sensor_id.to_string(),
            sensor_type,
            timestamp: Utc::now(),
            sequence: 0,
            data,
            dimensions: vec![],
            unit: String::new(),
            sample_rate: 0.0,
            quality: 1.0,
            position: None,
            orientation: None,
        }
    }
    
    pub fn as_vector(&self) -> DVector<f64> {
        DVector::from_vec(self.data.clone())
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Trait for all sensors
#[async_trait]
pub trait Sensor: Send + Sync {
    /// Get sensor unique identifier
    fn id(&self) -> &str;
    
    /// Get sensor type
    fn sensor_type(&self) -> SensorType;
    
    /// Get current status
    fn status(&self) -> SensorStatus;
    
    /// Connect to sensor hardware
    async fn connect(&mut self) -> Result<()>;
    
    /// Disconnect from sensor
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Perform calibration
    async fn calibrate(&mut self) -> Result<CalibrationData>;
    
    /// Read raw data from sensor
    async fn read(&mut self) -> Result<SensorReading>;
    
    /// Get sample rate in Hz
    fn sample_rate(&self) -> f64;
    
    /// Set sample rate
    fn set_sample_rate(&mut self, rate: f64) -> Result<()>;
    
    /// Get sensor configuration
    fn config(&self) -> serde_json::Value;
    
    /// Update sensor configuration
    fn set_config(&mut self, config: serde_json::Value) -> Result<()>;
}

/// Sensor health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHealth {
    pub sensor_id: String,
    pub status: SensorStatus,
    pub uptime_seconds: u64,
    pub readings_count: u64,
    pub error_count: u64,
    pub last_error: Option<String>,
    pub signal_quality: f32,
    pub noise_level: f64,
    pub temperature: Option<f64>,
    pub battery_level: Option<f32>,
}
