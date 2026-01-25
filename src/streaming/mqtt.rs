// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! MQTT client for streaming data

use anyhow::{anyhow, Result};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, Transport};
use serde::Serialize;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::StreamingConfig;

/// MQTT client wrapper
pub struct MqttClient {
    client: AsyncClient,
    config: MqttConfig,
    connected: RwLock<bool>,
}

#[derive(Clone)]
pub struct MqttConfig {
    pub broker: String,
    pub port: u16,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
    pub keep_alive_secs: u64,
    pub reconnect_interval_ms: u64,
}

impl MqttClient {
    pub async fn new(config: &StreamingConfig) -> Result<Self> {
        let mqtt_config = MqttConfig {
            broker: config.mqtt_broker.clone(),
            port: config.mqtt_port,
            client_id: config.mqtt_client_id.clone(),
            username: config.mqtt_username.clone(),
            password: config.mqtt_password.clone(),
            use_tls: config.mqtt_use_tls,
            keep_alive_secs: 30,
            reconnect_interval_ms: 5000,
        };
        
        let mut options = MqttOptions::new(
            &mqtt_config.client_id,
            &mqtt_config.broker,
            mqtt_config.port,
        );
        
        options.set_keep_alive(Duration::from_secs(mqtt_config.keep_alive_secs));
        
        if let (Some(username), Some(password)) = (&mqtt_config.username, &mqtt_config.password) {
            options.set_credentials(username, password);
        }
        
        if mqtt_config.use_tls {
            // Configure TLS
            // Note: In production, load proper certificates
            options.set_transport(Transport::Tcp);
        }
        
        let (client, mut eventloop) = AsyncClient::new(options, 100);
        
        // Spawn eventloop handler
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        info!("MQTT connected");
                    }
                    Ok(Event::Incoming(Packet::Publish(msg))) => {
                        debug!("MQTT received: {:?}", msg.topic);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("MQTT error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });
        
        Ok(Self {
            client,
            config: mqtt_config,
            connected: RwLock::new(false),
        })
    }
    
    pub async fn connect(&self) -> Result<()> {
        // Connection is handled by eventloop
        *self.connected.write().await = true;
        info!("MQTT client initialized for {}:{}", self.config.broker, self.config.port);
        Ok(())
    }
    
    pub async fn publish<T: Serialize>(&self, topic: &str, payload: &T) -> Result<()> {
        let json = serde_json::to_vec(payload)?;
        
        self.client.publish(topic, QoS::AtLeastOnce, false, json)
            .await
            .map_err(|e| anyhow!("MQTT publish failed: {}", e))?;
        
        Ok(())
    }
    
    pub async fn publish_raw(&self, topic: &str, payload: &[u8]) -> Result<()> {
        self.client.publish(topic, QoS::AtLeastOnce, false, payload.to_vec())
            .await
            .map_err(|e| anyhow!("MQTT publish failed: {}", e))?;
        
        Ok(())
    }
    
    pub async fn subscribe(&self, topic: &str) -> Result<()> {
        self.client.subscribe(topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| anyhow!("MQTT subscribe failed: {}", e))?;
        
        info!("Subscribed to MQTT topic: {}", topic);
        Ok(())
    }
    
    pub async fn disconnect(&self) -> Result<()> {
        self.client.disconnect()
            .await
            .map_err(|e| anyhow!("MQTT disconnect failed: {}", e))?;
        
        *self.connected.write().await = false;
        Ok(())
    }
    
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

/// MQTT message
#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: u8,
    pub retain: bool,
}
