// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! WebSocket server for real-time streaming

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{info, warn, error, debug};

use crate::sensors::SensorReading;
use crate::detection::Detection;

/// WebSocket server
pub struct WebSocketServer {
    port: u16,
    max_clients: usize,
    clients: Arc<RwLock<HashMap<String, ClientHandle>>>,
    broadcast_tx: broadcast::Sender<WebSocketMessage>,
}

struct ClientHandle {
    addr: SocketAddr,
    subscriptions: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum WebSocketMessage {
    SensorReading(String),  // JSON
    Detection(String),      // JSON
    System(String),         // System message
}

impl WebSocketServer {
    pub fn new(port: u16, max_clients: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1000);
        
        Self {
            port,
            max_clients,
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }
    
    pub async fn start(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("WebSocket server listening on ws://{}", addr);
        
        let clients = self.clients.clone();
        let max_clients = self.max_clients;
        let broadcast_tx = self.broadcast_tx.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                let client_count = clients.read().await.len();
                                if client_count >= max_clients {
                                    warn!("Max clients reached, rejecting connection from {}", addr);
                                    continue;
                                }
                                
                                let clients = clients.clone();
                                let broadcast_rx = broadcast_tx.subscribe();
                                
                                tokio::spawn(handle_connection(stream, addr, clients, broadcast_rx));
                            }
                            Err(e) => {
                                error!("Accept error: {}", e);
                            }
                        }
                    }
                    _ = shutdown.recv() => {
                        info!("WebSocket server shutting down");
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub async fn broadcast<T: Serialize>(&self, data: &T) -> Result<()> {
        let json = serde_json::to_string(data)?;
        let _ = self.broadcast_tx.send(WebSocketMessage::SensorReading(json));
        Ok(())
    }
    
    pub async fn broadcast_detection(&self, detection: &Detection) -> Result<()> {
        let json = serde_json::to_string(detection)?;
        let _ = self.broadcast_tx.send(WebSocketMessage::Detection(json));
        Ok(())
    }
    
    pub async fn send_to_client(&self, client_id: &str, message: &str) -> Result<()> {
        let _ = self.broadcast_tx.send(WebSocketMessage::System(message.to_string()));
        Ok(())
    }
    
    pub async fn get_client_count(&self) -> usize {
        self.clients.read().await.len()
    }
    
    pub async fn get_client_addrs(&self) -> Vec<SocketAddr> {
        self.clients.read().await
            .values()
            .map(|c| c.addr)
            .collect()
    }
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, ClientHandle>>>,
    mut broadcast_rx: broadcast::Receiver<WebSocketMessage>,
) {
    let client_id = uuid::Uuid::new_v4().to_string();
    
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed for {}: {}", addr, e);
            return;
        }
    };
    
    info!("New WebSocket connection from {} (id: {})", addr, client_id);
    
    // Register client
    {
        let mut clients = clients.write().await;
        clients.insert(client_id.clone(), ClientHandle {
            addr,
            subscriptions: vec!["*".to_string()],  // Subscribe to all by default
        });
    }
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    
    // Send welcome message
    let welcome = serde_json::json!({
        "type": "welcome",
        "client_id": client_id,
        "server": "GlowBarn",
        "version": env!("CARGO_PKG_VERSION"),
    });
    
    if let Err(e) = ws_sender.send(Message::Text(welcome.to_string().into())).await {
        warn!("Failed to send welcome: {}", e);
    }
    
    // Handle messages
    loop {
        tokio::select! {
            // Incoming messages from client
            msg = ws_receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received from {}: {}", addr, text);
                        
                        // Handle commands
                        if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(cmd_type) = cmd.get("type").and_then(|v| v.as_str()) {
                                match cmd_type {
                                    "ping" => {
                                        let pong = serde_json::json!({"type": "pong"});
                                        let _ = ws_sender.send(Message::Text(pong.to_string().into())).await;
                                    }
                                    "subscribe" => {
                                        if let Some(topic) = cmd.get("topic").and_then(|v| v.as_str()) {
                                            let mut clients = clients.write().await;
                                            if let Some(client) = clients.get_mut(&client_id) {
                                                client.subscriptions.push(topic.to_string());
                                            }
                                        }
                                    }
                                    "unsubscribe" => {
                                        if let Some(topic) = cmd.get("topic").and_then(|v| v.as_str()) {
                                            let mut clients = clients.write().await;
                                            if let Some(client) = clients.get_mut(&client_id) {
                                                client.subscriptions.retain(|s| s != topic);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket closed by client {}", addr);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = ws_sender.send(Message::Pong(data)).await;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error from {}: {}", addr, e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }
            
            // Outgoing broadcasts
            msg = broadcast_rx.recv() => {
                match msg {
                    Ok(WebSocketMessage::SensorReading(json)) => {
                        let wrapper = serde_json::json!({
                            "type": "reading",
                            "data": serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default()
                        });
                        if let Err(e) = ws_sender.send(Message::Text(wrapper.to_string().into())).await {
                            warn!("Failed to send to {}: {}", addr, e);
                            break;
                        }
                    }
                    Ok(WebSocketMessage::Detection(json)) => {
                        let wrapper = serde_json::json!({
                            "type": "detection",
                            "data": serde_json::from_str::<serde_json::Value>(&json).unwrap_or_default()
                        });
                        if let Err(e) = ws_sender.send(Message::Text(wrapper.to_string().into())).await {
                            warn!("Failed to send to {}: {}", addr, e);
                            break;
                        }
                    }
                    Ok(WebSocketMessage::System(msg)) => {
                        let wrapper = serde_json::json!({
                            "type": "system",
                            "message": msg
                        });
                        let _ = ws_sender.send(Message::Text(wrapper.to_string().into())).await;
                    }
                    Err(_) => {}
                }
            }
        }
    }
    
    // Remove client
    {
        let mut clients = clients.write().await;
        clients.remove(&client_id);
    }
    
    info!("WebSocket client {} disconnected", addr);
}
