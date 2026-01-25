//! RF sensors - SDR, spectrum analyzers, WiFi scanners

use async_trait::async_trait;
use anyhow::{Result, bail};
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Software-Defined Radio receiver
pub struct SDRSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    center_frequency: f64,  // Hz
    bandwidth: f64,  // Hz
    gain: f64,  // dB
}

impl SDRSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 2_400_000.0,  // 2.4 MSPS typical
            sequence: 0,
            center_frequency: 100_000_000.0,  // 100 MHz
            bandwidth: 2_000_000.0,  // 2 MHz
            gain: 30.0,
        }
    }
}

#[async_trait]
impl Sensor for SDRSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::SDRReceiver }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: -100.0,  // dBm
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("SDR calibration, center: {} MHz", self.center_frequency / 1e6),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "center_frequency": self.center_frequency,
            "bandwidth": self.bandwidth,
            "gain": self.gain
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(f) = config.get("center_frequency").and_then(|v| v.as_f64()) {
            self.center_frequency = f;
        }
        if let Some(b) = config.get("bandwidth").and_then(|v| v.as_f64()) {
            self.bandwidth = b;
        }
        if let Some(g) = config.get("gain").and_then(|v| v.as_f64()) {
            self.gain = g;
        }
        Ok(())
    }
}

/// RF Spectrum Analyzer
pub struct SpectrumAnalyzerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    start_frequency: f64,
    stop_frequency: f64,
    rbw: f64,  // Resolution bandwidth
    num_points: usize,
}

impl SpectrumAnalyzerSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 10.0,  // Sweeps per second
            sequence: 0,
            start_frequency: 1_000_000.0,  // 1 MHz
            stop_frequency: 6_000_000_000.0,  // 6 GHz
            rbw: 10_000.0,  // 10 kHz
            num_points: 1024,
        }
    }
}

#[async_trait]
impl Sensor for SpectrumAnalyzerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::SpectrumAnalyzer }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: -110.0,
            timestamp: Utc::now(),
            temperature: None,
            notes: format!("Spectrum analyzer {} MHz - {} GHz",
                self.start_frequency / 1e6, self.stop_frequency / 1e9),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "start_frequency": self.start_frequency,
            "stop_frequency": self.stop_frequency,
            "rbw": self.rbw,
            "num_points": self.num_points
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(f) = config.get("start_frequency").and_then(|v| v.as_f64()) {
            self.start_frequency = f;
        }
        if let Some(f) = config.get("stop_frequency").and_then(|v| v.as_f64()) {
            self.stop_frequency = f;
        }
        if let Some(r) = config.get("rbw").and_then(|v| v.as_f64()) {
            self.rbw = r;
        }
        Ok(())
    }
}

/// WiFi scanner
pub struct WiFiScannerSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
    bands: WiFiBands,
}

#[derive(Clone, Copy)]
pub struct WiFiBands {
    pub ghz_2_4: bool,
    pub ghz_5: bool,
    pub ghz_6: bool,
}

impl Default for WiFiBands {
    fn default() -> Self {
        Self { ghz_2_4: true, ghz_5: true, ghz_6: false }
    }
}

impl WiFiScannerSensor {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            status: SensorStatus::Disconnected,
            sample_rate: 0.5,  // Scans per second
            sequence: 0,
            bands: WiFiBands::default(),
        }
    }
}

#[async_trait]
impl Sensor for WiFiScannerSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::WiFiScanner }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: -95.0,
            timestamp: Utc::now(),
            temperature: None,
            notes: "WiFi scanner calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "bands_2_4ghz": self.bands.ghz_2_4,
            "bands_5ghz": self.bands.ghz_5,
            "bands_6ghz": self.bands.ghz_6
        })
    }
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(b) = config.get("bands_2_4ghz").and_then(|v| v.as_bool()) {
            self.bands.ghz_2_4 = b;
        }
        if let Some(b) = config.get("bands_5ghz").and_then(|v| v.as_bool()) {
            self.bands.ghz_5 = b;
        }
        if let Some(b) = config.get("bands_6ghz").and_then(|v| v.as_bool()) {
            self.bands.ghz_6 = b;
        }
        Ok(())
    }
}

/// EMI detector for electromagnetic interference
pub struct EMIDetectorSensor {
    id: String,
    status: SensorStatus,
    sample_rate: f64,
    sequence: u64,
}

impl EMIDetectorSensor {
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
impl Sensor for EMIDetectorSensor {
    fn id(&self) -> &str { &self.id }
    fn sensor_type(&self) -> SensorType { SensorType::EMIDetector }
    fn status(&self) -> SensorStatus { self.status }
    
    async fn connect(&mut self) -> Result<()> { self.status = SensorStatus::Connected; Ok(()) }
    async fn disconnect(&mut self) -> Result<()> { self.status = SensorStatus::Disconnected; Ok(()) }
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Active;
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: -80.0,
            timestamp: Utc::now(),
            temperature: None,
            notes: "EMI detector calibration".to_string(),
            signature: vec![],
        })
    }
    async fn read(&mut self) -> Result<SensorReading> { bail!("Hardware not connected") }
    fn sample_rate(&self) -> f64 { self.sample_rate }
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> { self.sample_rate = rate; Ok(()) }
    fn config(&self) -> serde_json::Value { serde_json::json!({}) }
    fn set_config(&mut self, _config: serde_json::Value) -> Result<()> { Ok(()) }
}
