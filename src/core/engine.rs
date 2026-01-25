//! Main detection engine - simplified for initial compilation

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::info;

use crate::config::Config;
use super::SystemState;

/// Main GlowBarn engine - simplified for initial build
pub struct Engine {
    pub config: Arc<Config>,
    state: Arc<RwLock<SystemState>>,
    start_time: Option<Instant>,
}

impl Engine {
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);
        
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(SystemState::default())),
            start_time: None,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting GlowBarn engine...");
        self.start_time = Some(Instant::now());
        
        {
            let mut state = self.state.write().await;
            state.running = true;
        }
        
        info!("GlowBarn engine started");
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping GlowBarn engine...");
        
        {
            let mut state = self.state.write().await;
            state.running = false;
        }
        
        info!("GlowBarn engine stopped");
        Ok(())
    }
    
    pub async fn state(&self) -> SystemState {
        self.state.read().await.clone()
    }
    
    pub fn uptime(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }
}
