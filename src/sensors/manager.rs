// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Sensor manager - coordinates all sensors

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use anyhow::Result;
use tracing::{info, warn, error, debug};

use super::{Sensor, SensorReading, SensorType, SensorStatus, SensorHealth};
use super::simulator::SensorSimulator;
use crate::config::Config;
use crate::core::EventBus;

/// Manages all sensors in the system
pub struct SensorManager {
    config: Arc<Config>,
    sensors: RwLock<HashMap<String, Box<dyn Sensor>>>,
    health: RwLock<HashMap<String, SensorHealth>>,
    event_bus: Arc<EventBus>,
    demo_mode: bool,
}

impl SensorManager {
    pub async fn new(config: Arc<Config>, event_bus: Arc<EventBus>, demo_mode: bool) -> Result<Self> {
        let manager = Self {
            config,
            sensors: RwLock::new(HashMap::new()),
            health: RwLock::new(HashMap::new()),
            event_bus,
            demo_mode,
        };
        
        if demo_mode {
            manager.add_demo_sensors().await?;
        }
        
        Ok(manager)
    }
    
    async fn add_demo_sensors(&self) -> Result<()> {
        info!("Adding demo sensors...");
        
        let demo_configs = vec![
            ("thermal-array-1", SensorType::ThermalArray, 10.0),
            ("accel-1", SensorType::Accelerometer, 100.0),
            ("emf-probe-1", SensorType::EMFProbe, 50.0),
            ("infrasound-1", SensorType::Infrasound, 48000.0),
            ("ultrasonic-1", SensorType::Ultrasonic, 192000.0),
            ("geiger-1", SensorType::GeigerCounter, 1.0),
            ("ion-counter-1", SensorType::IonCounter, 1.0),
            ("rf-scanner-1", SensorType::SDRReceiver, 1000.0),
            ("qrng-1", SensorType::QRNG, 1000.0),
            ("flux-gate-1", SensorType::FluxGate, 100.0),
            ("spectrometer-1", SensorType::Spectrometer, 10.0),
            ("barometer-1", SensorType::Barometer, 1.0),
            ("static-meter-1", SensorType::StaticMeter, 10.0),
            ("laser-grid-1", SensorType::LaserGrid, 60.0),
        ];
        
        for (id, sensor_type, sample_rate) in demo_configs {
            let simulator = SensorSimulator::new(id, sensor_type, sample_rate);
            self.add_sensor(Box::new(simulator)).await?;
        }
        
        Ok(())
    }
    
    pub async fn add_sensor(&self, sensor: Box<dyn Sensor>) -> Result<()> {
        let id = sensor.id().to_string();
        let sensor_type = sensor.sensor_type();
        
        let mut sensors = self.sensors.write().await;
        sensors.insert(id.clone(), sensor);
        
        let mut health = self.health.write().await;
        health.insert(id.clone(), SensorHealth {
            sensor_id: id.clone(),
            status: SensorStatus::Disconnected,
            uptime_seconds: 0,
            readings_count: 0,
            error_count: 0,
            last_error: None,
            signal_quality: 0.0,
            noise_level: 0.0,
            temperature: None,
            battery_level: None,
        });
        
        info!("Added sensor: {} ({:?})", id, sensor_type);
        Ok(())
    }
    
    pub async fn remove_sensor(&self, id: &str) -> Result<()> {
        let mut sensors = self.sensors.write().await;
        if let Some(mut sensor) = sensors.remove(id) {
            sensor.disconnect().await?;
        }
        
        let mut health = self.health.write().await;
        health.remove(id);
        
        info!("Removed sensor: {}", id);
        Ok(())
    }
    
    pub async fn active_count(&self) -> usize {
        let sensors = self.sensors.read().await;
        sensors.values().filter(|s| s.status() == SensorStatus::Active).count()
    }
    
    pub async fn get_health(&self, id: &str) -> Option<SensorHealth> {
        let health = self.health.read().await;
        health.get(id).cloned()
    }
    
    pub async fn get_all_health(&self) -> Vec<SensorHealth> {
        let health = self.health.read().await;
        health.values().cloned().collect()
    }
    
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("Starting sensor manager...");
        
        // Connect all sensors
        {
            let mut sensors = self.sensors.write().await;
            for (id, sensor) in sensors.iter_mut() {
                match sensor.connect().await {
                    Ok(_) => {
                        info!("Connected sensor: {}", id);
                        if let Err(e) = sensor.calibrate().await {
                            warn!("Calibration failed for {}: {}", id, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect sensor {}: {}", id, e);
                    }
                }
            }
        }
        
        // Main reading loop
        let mut read_interval = interval(Duration::from_millis(10));  // 100Hz base rate
        
        loop {
            tokio::select! {
                _ = read_interval.tick() => {
                    self.read_all_sensors().await;
                }
                _ = shutdown.recv() => {
                    info!("Sensor manager shutting down...");
                    break;
                }
            }
        }
        
        // Disconnect all sensors
        {
            let mut sensors = self.sensors.write().await;
            for (id, sensor) in sensors.iter_mut() {
                if let Err(e) = sensor.disconnect().await {
                    warn!("Error disconnecting {}: {}", id, e);
                }
            }
        }
        
        Ok(())
    }
    
    async fn read_all_sensors(&self) {
        let mut sensors = self.sensors.write().await;
        let mut health = self.health.write().await;
        
        for (id, sensor) in sensors.iter_mut() {
            if sensor.status() != SensorStatus::Active {
                continue;
            }
            
            match sensor.read().await {
                Ok(reading) => {
                    // Update health
                    if let Some(h) = health.get_mut(id) {
                        h.readings_count += 1;
                        h.signal_quality = reading.quality;
                    }
                    
                    // Publish reading
                    self.event_bus.publish_reading(reading);
                }
                Err(e) => {
                    if let Some(h) = health.get_mut(id) {
                        h.error_count += 1;
                        h.last_error = Some(e.to_string());
                    }
                    debug!("Read error for {}: {}", id, e);
                }
            }
        }
    }
}
