// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Sensor simulator for demo/testing

use async_trait::async_trait;
use anyhow::Result;
use rand::prelude::*;
use rand_distr::{Normal, Uniform};
use std::f64::consts::PI;
use chrono::Utc;

use super::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData};

/// Simulates realistic sensor data for testing
pub struct SensorSimulator {
    id: String,
    sensor_type: SensorType,
    sample_rate: f64,
    status: SensorStatus,
    sequence: u64,
    rng: rand::rngs::StdRng,
    
    // Simulation state
    time: f64,
    anomaly_probability: f64,
    noise_level: f64,
    drift: f64,
}

impl SensorSimulator {
    pub fn new(id: &str, sensor_type: SensorType, sample_rate: f64) -> Self {
        Self {
            id: id.to_string(),
            sensor_type,
            sample_rate,
            status: SensorStatus::Disconnected,
            sequence: 0,
            rng: rand::rngs::StdRng::from_entropy(),
            time: 0.0,
            anomaly_probability: 0.02,
            noise_level: 0.1,
            drift: 0.0,
        }
    }
    
    fn generate_data(&mut self) -> Vec<f64> {
        self.time += 1.0 / self.sample_rate;
        self.drift += self.rng.gen_range(-0.001..0.001);
        
        match self.sensor_type {
            SensorType::ThermalArray => self.generate_thermal_array(),
            SensorType::ThermalImager => self.generate_thermal_imager(),
            SensorType::Accelerometer => self.generate_accelerometer(),
            SensorType::Geophone => self.generate_geophone(),
            SensorType::EMFProbe => self.generate_emf(),
            SensorType::FluxGate => self.generate_fluxgate(),
            SensorType::Infrasound => self.generate_infrasound(),
            SensorType::Ultrasonic => self.generate_ultrasonic(),
            SensorType::GeigerCounter => self.generate_geiger(),
            SensorType::IonCounter => self.generate_ion_counter(),
            SensorType::SDRReceiver => self.generate_rf_spectrum(),
            SensorType::QRNG => self.generate_qrng(),
            SensorType::Spectrometer => self.generate_spectrometer(),
            SensorType::Barometer => self.generate_barometer(),
            SensorType::StaticMeter => self.generate_static(),
            SensorType::LaserGrid => self.generate_laser_grid(),
            SensorType::CapacitiveSensor => self.generate_capacitive(),
            _ => self.generate_generic(),
        }
    }
    
    fn generate_thermal_array(&mut self) -> Vec<f64> {
        let mut data = vec![0.0; 64]; // 8x8 grid
        let ambient = 22.0 + self.drift;
        
        for i in 0..64 {
            let x = (i % 8) as f64;
            let y = (i / 8) as f64;
            
            // Base ambient temperature with slight gradient
            let mut temp = ambient + (y - 4.0) * 0.1;
            
            // Add noise
            temp += self.rng.sample::<f64, _>(Normal::new(0.0, 0.3).unwrap());
            
            // Random hot/cold spots (anomalies)
            if self.rng.gen::<f64>() < self.anomaly_probability * 0.5 {
                let anomaly_x = self.rng.gen_range(0..8) as f64;
                let anomaly_y = self.rng.gen_range(0..8) as f64;
                let dist = ((x - anomaly_x).powi(2) + (y - anomaly_y).powi(2)).sqrt();
                if dist < 2.0 {
                    let intensity = self.rng.gen_range(5.0..15.0);
                    temp += intensity * (-dist / 1.5).exp();
                }
            }
            
            data[i] = temp;
        }
        
        data
    }
    
    fn generate_thermal_imager(&mut self) -> Vec<f64> {
        // 80x60 thermal image
        let mut data = vec![0.0; 4800];
        let ambient = 22.0 + self.drift;
        
        for i in 0..4800 {
            let x = (i % 80) as f64 / 80.0;
            let y = (i / 80) as f64 / 60.0;
            
            let mut temp = ambient + (y - 0.5) * 2.0;
            temp += self.rng.sample::<f64, _>(Normal::new(0.0, 0.2).unwrap());
            
            // Thermal patterns
            if self.rng.gen::<f64>() < self.anomaly_probability * 0.1 {
                let cx = self.rng.gen::<f64>();
                let cy = self.rng.gen::<f64>();
                let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                if dist < 0.2 {
                    temp += self.rng.gen_range(3.0..10.0) * (-dist / 0.1).exp();
                }
            }
            
            data[i] = temp;
        }
        
        data
    }
    
    fn generate_accelerometer(&mut self) -> Vec<f64> {
        let mut data = vec![0.0; 3];
        
        // Gravity on Z axis
        data[0] = self.rng.sample::<f64, _>(Normal::new(0.0, 0.01).unwrap());
        data[1] = self.rng.sample::<f64, _>(Normal::new(0.0, 0.01).unwrap());
        data[2] = 1.0 + self.rng.sample::<f64, _>(Normal::new(0.0, 0.01).unwrap());
        
        // Vibration events
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let axis = self.rng.gen_range(0..3);
            data[axis] += self.rng.gen_range(0.1..0.5) * self.rng.gen_range(-1.0..1.0);
        }
        
        data
    }
    
    fn generate_geophone(&mut self) -> Vec<f64> {
        let samples = (self.sample_rate / 10.0) as usize;
        let mut data = vec![0.0; samples];
        
        for i in 0..samples {
            let t = i as f64 / self.sample_rate;
            
            // Background microseismic noise
            data[i] = self.rng.sample::<f64, _>(Normal::new(0.0, 1e-7).unwrap());
            
            // Low frequency earth movement (0.1-1 Hz)
            data[i] += 5e-7 * (2.0 * PI * 0.15 * (self.time + t)).sin();
        }
        
        // Seismic event
        if self.rng.gen::<f64>() < self.anomaly_probability * 0.5 {
            let event_pos = self.rng.gen_range(0..samples);
            let freq = self.rng.gen_range(2.0..20.0);
            let amp = self.rng.gen_range(1e-5..1e-4);
            
            for i in 0..samples {
                let dist = (i as i32 - event_pos as i32).abs() as f64;
                let envelope = (-dist / (samples as f64 * 0.1)).exp();
                data[i] += amp * envelope * (2.0 * PI * freq * i as f64 / self.sample_rate).sin();
            }
        }
        
        data
    }
    
    fn generate_emf(&mut self) -> Vec<f64> {
        // EMF reading in milliGauss
        let base = 0.5 + self.drift.abs() * 10.0;
        let mut value = base + self.rng.sample::<f64, _>(Normal::new(0.0, 0.1).unwrap());
        
        // AC hum (60Hz)
        value += 0.3 * (2.0 * PI * 60.0 * self.time).sin();
        
        // EMF spike
        if self.rng.gen::<f64>() < self.anomaly_probability {
            value += self.rng.gen_range(5.0..50.0);
        }
        
        vec![value.abs()]
    }
    
    fn generate_fluxgate(&mut self) -> Vec<f64> {
        // 3-axis magnetometer in microTesla
        let earth_field = 50.0; // Earth's magnetic field ~50 µT
        
        vec![
            earth_field * 0.3 + self.rng.sample::<f64, _>(Normal::new(0.0, 0.1).unwrap()),
            earth_field * 0.1 + self.rng.sample::<f64, _>(Normal::new(0.0, 0.1).unwrap()),
            earth_field * 0.9 + self.rng.sample::<f64, _>(Normal::new(0.0, 0.1).unwrap()),
        ]
    }
    
    fn generate_infrasound(&mut self) -> Vec<f64> {
        let samples = (self.sample_rate / 10.0) as usize;
        let mut data = vec![0.0; samples];
        
        for i in 0..samples {
            let t = i as f64 / self.sample_rate;
            
            // Very low frequency content
            data[i] = self.rng.sample::<f64, _>(Normal::new(0.0, 0.0001).unwrap());
            
            // Infrasonic tones (1-20 Hz)
            data[i] += 0.0003 * (2.0 * PI * 7.83 * (self.time + t)).sin();  // Schumann resonance
            data[i] += 0.0002 * (2.0 * PI * 3.5 * (self.time + t)).sin();
        }
        
        // Infrasonic event
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let freq = self.rng.gen_range(1.0..15.0);
            let amp = self.rng.gen_range(0.001..0.01);
            for i in 0..samples {
                let t = i as f64 / self.sample_rate;
                data[i] += amp * (2.0 * PI * freq * (self.time + t)).sin();
            }
        }
        
        data
    }
    
    fn generate_ultrasonic(&mut self) -> Vec<f64> {
        let samples = (self.sample_rate / 10.0) as usize;
        let mut data = vec![0.0; samples];
        
        for i in 0..samples {
            // High frequency noise floor
            data[i] = self.rng.sample::<f64, _>(Normal::new(0.0, 0.001).unwrap());
        }
        
        // Ultrasonic tone burst
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let freq = self.rng.gen_range(25000.0..80000.0);
            let start = self.rng.gen_range(0..samples/2);
            let duration = self.rng.gen_range(100..500);
            let amp = self.rng.gen_range(0.01..0.1);
            
            for i in start..(start + duration).min(samples) {
                let t = i as f64 / self.sample_rate;
                let envelope = (-((i as i64 - start as i64 - duration as i64/2).pow(2)) as f64 / (duration as f64).powi(2) * 10.0).exp();
                data[i] += amp * envelope * (2.0 * PI * freq * t).sin();
            }
        }
        
        data
    }
    
    fn generate_geiger(&mut self) -> Vec<f64> {
        // Counts per minute (CPM)
        // Normal background: 15-30 CPM
        let base_cpm = 20.0 + self.drift * 5.0;
        
        // Poisson-distributed counts
        let lambda = base_cpm / 60.0; // events per second
        let mut count = 0;
        let mut p = 1.0;
        let l = (-lambda).exp();
        
        loop {
            count += 1;
            p *= self.rng.gen::<f64>();
            if p <= l {
                break;
            }
        }
        count -= 1;
        
        // Radiation spike
        if self.rng.gen::<f64>() < self.anomaly_probability * 0.3 {
            count += self.rng.gen_range(5..50);
        }
        
        vec![count as f64]
    }
    
    fn generate_ion_counter(&mut self) -> Vec<f64> {
        // Ions per cm³
        let positive = 500.0 + self.rng.sample::<f64, _>(Normal::new(0.0, 50.0).unwrap());
        let negative = 600.0 + self.rng.sample::<f64, _>(Normal::new(0.0, 50.0).unwrap());
        
        vec![positive.max(0.0), negative.max(0.0)]
    }
    
    fn generate_rf_spectrum(&mut self) -> Vec<f64> {
        // 256 frequency bins from 1MHz to 1GHz
        let bins = 256;
        let mut data = vec![0.0; bins];
        
        for i in 0..bins {
            // Noise floor around -90 dBm
            data[i] = -90.0 + self.rng.sample::<f64, _>(Normal::new(0.0, 3.0).unwrap());
        }
        
        // Known signals (FM radio, WiFi, etc.)
        let signals = [
            (88, -50.0),  // FM radio
            (150, -60.0), // WiFi 2.4GHz
            (200, -55.0), // WiFi 5GHz
        ];
        
        for (bin, strength) in signals {
            if bin < bins {
                data[bin] = strength + self.rng.gen_range(-5.0..5.0);
                if bin > 0 { data[bin-1] = strength - 10.0; }
                if bin < bins-1 { data[bin+1] = strength - 10.0; }
            }
        }
        
        // Anomalous signal
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let bin = self.rng.gen_range(0..bins);
            data[bin] = self.rng.gen_range(-40.0..-20.0);
        }
        
        data
    }
    
    fn generate_qrng(&mut self) -> Vec<f64> {
        // Quantum random numbers (should be perfectly uniform)
        let samples = 100;
        let mut data = vec![0.0; samples];
        
        for i in 0..samples {
            data[i] = self.rng.gen::<f64>();
        }
        
        // Introduce subtle bias for anomaly
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let bias = self.rng.gen_range(-0.1..0.1);
            for d in &mut data {
                *d = (*d + bias).clamp(0.0, 1.0);
            }
        }
        
        data
    }
    
    fn generate_spectrometer(&mut self) -> Vec<f64> {
        // 512 wavelength bins (380nm - 780nm visible spectrum)
        let bins = 512;
        let mut data = vec![0.0; bins];
        
        for i in 0..bins {
            // Background light level
            data[i] = 0.1 + self.rng.sample::<f64, _>(Normal::new(0.0, 0.01).unwrap());
            
            // Natural daylight spectrum approximation
            let wavelength = 380.0 + (i as f64 / bins as f64) * 400.0;
            let peak = ((wavelength - 550.0) / 100.0).powi(2);
            data[i] += 0.5 * (-peak).exp();
        }
        
        data
    }
    
    fn generate_barometer(&mut self) -> Vec<f64> {
        // Pressure in hPa
        let pressure = 1013.25 + self.drift * 10.0;
        let pressure = pressure + self.rng.sample::<f64, _>(Normal::new(0.0, 0.1).unwrap());
        
        vec![pressure]
    }
    
    fn generate_static(&mut self) -> Vec<f64> {
        // Static electricity in V/m
        let mut field = 100.0 + self.rng.sample::<f64, _>(Normal::new(0.0, 20.0).unwrap());
        
        // Static discharge event
        if self.rng.gen::<f64>() < self.anomaly_probability {
            field += self.rng.gen_range(500.0..5000.0);
        }
        
        vec![field]
    }
    
    fn generate_laser_grid(&mut self) -> Vec<f64> {
        // 16 laser beams, value = beam intensity (0-1)
        let mut data = vec![1.0; 16];
        
        for d in &mut data {
            *d = 1.0 - self.rng.sample::<f64, _>(Normal::new(0.0, 0.02).unwrap()).abs();
        }
        
        // Beam interruption
        if self.rng.gen::<f64>() < self.anomaly_probability {
            let beam = self.rng.gen_range(0..16);
            data[beam] = self.rng.gen_range(0.0..0.3);
        }
        
        data
    }
    
    fn generate_capacitive(&mut self) -> Vec<f64> {
        // Proximity in arbitrary units
        let mut proximity = self.rng.sample::<f64, _>(Normal::new(0.0, 0.05).unwrap()).abs();
        
        // Object approaching
        if self.rng.gen::<f64>() < self.anomaly_probability {
            proximity += self.rng.gen_range(0.2..1.0);
        }
        
        vec![proximity]
    }
    
    fn generate_generic(&mut self) -> Vec<f64> {
        vec![self.rng.sample::<f64, _>(Normal::new(0.0, 1.0).unwrap())]
    }
}

#[async_trait]
impl Sensor for SensorSimulator {
    fn id(&self) -> &str {
        &self.id
    }
    
    fn sensor_type(&self) -> SensorType {
        self.sensor_type
    }
    
    fn status(&self) -> SensorStatus {
        self.status
    }
    
    async fn connect(&mut self) -> Result<()> {
        self.status = SensorStatus::Connected;
        Ok(())
    }
    
    async fn disconnect(&mut self) -> Result<()> {
        self.status = SensorStatus::Disconnected;
        Ok(())
    }
    
    async fn calibrate(&mut self) -> Result<CalibrationData> {
        self.status = SensorStatus::Calibrating;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        self.status = SensorStatus::Active;
        
        Ok(CalibrationData {
            offset: vec![0.0],
            scale: vec![1.0],
            noise_floor: self.noise_level,
            timestamp: Utc::now(),
            temperature: Some(25.0),
            notes: "Simulated calibration".to_string(),
            signature: vec![],
        })
    }
    
    async fn read(&mut self) -> Result<SensorReading> {
        let data = self.generate_data();
        self.sequence += 1;
        
        let unit = match self.sensor_type {
            SensorType::ThermalArray | SensorType::ThermalImager => "°C",
            SensorType::Accelerometer => "g",
            SensorType::Geophone => "m/s",
            SensorType::EMFProbe => "mG",
            SensorType::FluxGate => "µT",
            SensorType::GeigerCounter => "CPM",
            SensorType::Barometer => "hPa",
            SensorType::StaticMeter => "V/m",
            SensorType::SDRReceiver => "dBm",
            _ => "",
        };
        
        Ok(SensorReading {
            sensor_id: self.id.clone(),
            sensor_type: self.sensor_type,
            timestamp: Utc::now(),
            sequence: self.sequence,
            data,
            dimensions: vec![],
            unit: unit.to_string(),
            sample_rate: self.sample_rate,
            quality: 1.0 - self.noise_level as f32 * 0.5,
            position: None,
            orientation: None,
        })
    }
    
    fn sample_rate(&self) -> f64 {
        self.sample_rate
    }
    
    fn set_sample_rate(&mut self, rate: f64) -> Result<()> {
        self.sample_rate = rate;
        Ok(())
    }
    
    fn config(&self) -> serde_json::Value {
        serde_json::json!({
            "anomaly_probability": self.anomaly_probability,
            "noise_level": self.noise_level,
        })
    }
    
    fn set_config(&mut self, config: serde_json::Value) -> Result<()> {
        if let Some(ap) = config.get("anomaly_probability").and_then(|v| v.as_f64()) {
            self.anomaly_probability = ap;
        }
        if let Some(nl) = config.get("noise_level").and_then(|v| v.as_f64()) {
            self.noise_level = nl;
        }
        Ok(())
    }
}
