// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! GlowBarn - High-Performance Paranormal Detection Suite
//! 
//! A high-performance, cross-platform native application for paranormal investigation,
//! environmental monitoring, and multi-modal anomaly detection.
//!
//! Features:
//! - 50+ sensor types for comprehensive detection
//! - Real-time multi-sensor fusion (Bayesian, Dempster-Shafer)
//! - GPU-accelerated entropy analysis (10+ entropy measures)
//! - Advanced mathematical detection methods
//! - Military-grade AES-256-GCM encryption
//! - Cross-platform native GUI (Windows, macOS, Linux)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anyhow::Result;
use clap::Parser;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::path::PathBuf;

use glowbarn::{Config, VERSION};

/// GlowBarn - High-Performance Paranormal Detection Suite
#[derive(Parser, Debug)]
#[command(name = "glowbarn")]
#[command(author = "GlowBarn Project")]
#[command(version = VERSION)]
#[command(about = "High-performance paranormal detection and anomaly analysis")]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Run in headless mode (no GUI)
    #[arg(long)]
    headless: bool,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Enable trace-level logging
    #[arg(long)]
    trace: bool,

    /// Demo mode with simulated sensors
    #[arg(long)]
    demo: bool,

    /// WebSocket server port
    #[arg(long, default_value = "8765")]
    ws_port: u16,

    /// MQTT broker address
    #[arg(long)]
    mqtt_broker: Option<String>,

    /// Data output directory
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.trace {
        Level::TRACE
    } else if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(args.debug)
        .with_line_number(args.debug)
        .with_ansi(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🌟 GlowBarn v{} - High-Performance Paranormal Detection Suite", VERSION);
    info!("   Built with Rust for maximum performance and safety");

    // Load or create configuration
    let config_path = args.config.unwrap_or_else(Config::default_path);
    let mut config = Config::load_or_create(&config_path)?;
    
    // Override with command line args
    if args.demo {
        config.demo_mode = true;
    }
    if let Some(data_dir) = args.data_dir {
        config.data_dir = data_dir;
    }
    config.streaming.websocket_port = args.ws_port;
    if let Some(mqtt) = args.mqtt_broker {
        config.streaming.mqtt_enabled = true;
        config.streaming.mqtt_broker = mqtt;
    }

    info!("Configuration loaded from {:?}", config_path);
    info!("Demo mode: {}", config.demo_mode);

    if args.headless {
        // Run headless mode
        info!("Starting in headless mode...");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(run_headless(config))?;
    } else {
        // Run GUI application
        #[cfg(feature = "gui")]
        {
            info!("Starting visual console...");
            glowbarn::ui::run_gui(config)?;
        }
        
        #[cfg(not(feature = "gui"))]
        {
            anyhow::bail!("GUI feature not enabled. Build with --features gui or use --headless");
        }
    }

    Ok(())
}

/// Run the application in headless mode (no GUI)
async fn run_headless(config: Config) -> Result<()> {
    use glowbarn::{
        core::Engine,
        streaming::StreamingManager,
        db::Database,
    };
    use tokio::sync::broadcast;
    
    info!("Initializing headless mode...");
    
    // Initialize database
    let db_path = config.data_dir.join("glowbarn.db");
    let db = Database::open(&config.database)?;
    info!("Database opened at {:?}", db_path);
    
    // Create event channel for sensor data
    let (tx, _rx): (broadcast::Sender<String>, broadcast::Receiver<String>) = broadcast::channel(1000);
    
    // Initialize streaming if enabled
    let streaming = if config.streaming.websocket_enabled {
        let streaming = StreamingManager::new(config.streaming.clone());
        info!("Streaming manager initialized");
        Some(streaming)
    } else {
        None
    };
    
    // Initialize the core engine
    let engine = Engine::new(config.clone()).await?;
    info!("Core engine initialized");
    
    info!("🚀 GlowBarn running in headless mode");
    info!("   Press Ctrl+C to shutdown");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    info!("Shutdown signal received, cleaning up...");
    
    // Cleanup
    drop(streaming);
    drop(db);
    
    info!("GlowBarn shutdown complete");
    
    Ok(())
}