//! Radiation sensors - Geiger counters, scintillators, neutron detectors

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Geiger-Müller counter
pub struct GeigerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    tube_type: GeigerTubeType,
    conversion_factor: f64,  // CPM to µSv/h
}

#[derive(Clone, Copy)]
pub enum GeigerTubeType {
    SBM20,      // Russian pancake
    SI29BG,     // End-window
    LND712,     // Halogen-quenched
    J408Gamma,  // High-sensitivity gamma
    Custom,
}

impl GeigerSensor {
    pub fn new(id: &str, tube_type: GeigerTubeType) -> Self {
        let conversion_factor = match tube_type {
            GeigerTubeType::SBM20 => 0.0057,
            GeigerTubeType::SI29BG => 0.006315,
            GeigerTubeType::LND712 => 0.00833,
            GeigerTubeType::J408Gamma => 0.005,
            GeigerTubeType::Custom => 0.006,
        };
        
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
            tube_type,
            conversion_factor,
        }
    }
}

#[async_trait]
impl Sensor for GeigerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::GeigerCounter }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![self.conversion_factor],
            noise_floor: 5.0,  // ~5 CPM noise floor
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Geiger tube calibration, conversion: {} CPM/µSv/h", 1.0/self.conversion_factor),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"conversion_factor": self.conversion_factor}) }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(cf) = config.get("conversion_factor").and_then(|v| v.as_f64()) {
            self.conversion_factor = cf;
        }
        Ok(())
    }
}

/// Scintillation detector for gamma spectroscopy
pub struct ScintillatorSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    crystal_type: ScintillatorType,
    energy_calibration: (f64, f64),  // keV = a + b * channel
}

#[derive(Clone, Copy)]
pub enum ScintillatorType {
    NaI,      // Sodium Iodide
    CsI,      // Cesium Iodide
    BGO,      // Bismuth Germanate
    LaBr3,    // Lanthanum Bromide
    Plastic,  // Plastic scintillator
}

impl ScintillatorSensor {
    pub fn new(id: &str, crystal_type: ScintillatorType) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 100.0,  // Spectra per second
            sequence: 0,
            crystal_type,
            energy_calibration: (0.0, 3.0),  // Typical 3 keV/channel
        }
    }
}

#[async_trait]
impl Sensor for ScintillatorSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::Scintillator }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![self.energy_calibration.0],
            scale: vec![self.energy_calibration.1],
            noise_floor: 10.0,  // keV threshold
            timestamp: Utc::now(),
            temperature: Some(25.0),
            notes: "Scintillator energy calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "energy_calibration": self.energy_calibration
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(cal) = config.get("energy_calibration").and_then(|v| v.as_array()) {
            if cal.len() == 2 {
                let a = cal[0].as_f64().unwrap_or(0.0);
                let b = cal[1].as_f64().unwrap_or(3.0);
                self.energy_calibration = (a, b);
            }
        }
        Ok(())
    }
}

/// Neutron detector (He-3 or BF3 proportional counter)
pub struct NeutronSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl NeutronSensor {
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
impl Sensor for NeutronSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::NeutronDetector }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: 0.01,  // Very low background
            timestamp: Utc::now(),
            temperature: None,
            notes: "Neutron detector calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}

/// Dosimeter array for spatial radiation mapping
pub struct DosimeterArraySensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    num_dosimeters: usize,
}

impl DosimeterArraySensor {
    pub fn new(id: &str, num_dosimeters: usize) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 1.0,
            sequence: 0,
            num_dosimeters,
        }
    }
}

#[async_trait]
impl Sensor for DosimeterArraySensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::DosimeterArray }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0; self.num_dosimeters],
            scale: vec![1.0; self.num_dosimeters],
            noise_floor: 0.01,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("{}-element dosimeter array calibration", self.num_dosimeters),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({"num_dosimeters": self.num_dosimeters}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
