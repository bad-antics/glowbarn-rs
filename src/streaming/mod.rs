//! Streaming module - MQTT, WebSocket, and data export

mod mqtt;
mod websocket;
mod export;

pub use mqtt::*;
pub use websocket::*;
pub use export::*;

use std::sync::Arc;
use tokio::sync::broadcast;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Enable MQTT
    pub mqtt_enabled: bool,
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub mqtt_client_id: String,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub mqtt_use_tls: bool,
    
    /// Enable WebSocket server
    pub websocket_enabled: bool,
    pub websocket_port: u16,
    pub websocket_max_clients: usize,
    
    /// Enable data export
    pub export_enabled: bool,
    pub export_format: ExportFormat,
    pub export_path: String,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            mqtt_enabled: false,
            mqtt_broker: "localhost".to_string(),
            mqtt_port: 1883,
            mqtt_client_id: "glowbarn".to_string(),
            mqtt_username: None,
            mqtt_password: None,
            mqtt_use_tls: false,
            
            websocket_enabled: false,
            websocket_port: 8765,
            websocket_max_clients: 10,
            
            export_enabled: true,
            export_format: ExportFormat::Json,
            export_path: "./data".to_string(),
        }
    }
}

/// Export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExportFormat {
    Json,
    Csv,
    Binary,
    InfluxLineProtocol,
}

/// Streaming manager
pub struct StreamingManager {
    config: StreamingConfig,
    mqtt_client: Option<MqttClient>,
    websocket_server: Option<WebSocketServer>,
    exporter: DataExporter,
}

impl StreamingManager {
    pub async fn new(config: StreamingConfig) -> Result<Self> {
        let mqtt_client = if config.mqtt_enabled {
            Some(MqttClient::new(&config).await?)
        } else {
            None
        };
        
        let websocket_server = if config.websocket_enabled {
            Some(WebSocketServer::new(config.websocket_port, config.websocket_max_clients))
        } else {
            None
        };
        
        let exporter = DataExporter::new(&config.export_path, config.export_format)?;
        
        Ok(Self {
            config,
            mqtt_client,
            websocket_server,
            exporter,
        })
    }
    
    pub async fn start(&mut self, shutdown: broadcast::Receiver<()>) -> Result<()> {
        if let Some(ref mut mqtt) = self.mqtt_client {
            mqtt.connect().await?;
        }
        
        if let Some(ref mut ws) = self.websocket_server {
            ws.start(shutdown).await?;
        }
        
        Ok(())
    }
    
    pub async fn publish_reading(&self, reading: &crate::sensors::SensorReading) -> Result<()> {
        // MQTT
        if let Some(ref mqtt) = self.mqtt_client {
            let topic = format!("glowbarn/sensors/{}", reading.sensor_id);
            mqtt.publish(&topic, reading).await?;
        }
        
        // WebSocket
        if let Some(ref ws) = self.websocket_server {
            ws.broadcast(reading).await?;
        }
        
        // Export
        if self.config.export_enabled {
            self.exporter.export_reading(reading)?;
        }
        
        Ok(())
    }
    
    pub async fn publish_detection(&self, detection: &crate::detection::Detection) -> Result<()> {
        // MQTT
        if let Some(ref mqtt) = self.mqtt_client {
            mqtt.publish("glowbarn/detections", detection).await?;
        }
        
        // WebSocket
        if let Some(ref ws) = self.websocket_server {
            ws.broadcast_detection(detection).await?;
        }
        
        // Export
        if self.config.export_enabled {
            self.exporter.export_detection(detection)?;
        }
        
        Ok(())
    }
}
