// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! GlowBarn - High-Performance Paranormal Detection Suite
//!
//! A comprehensive Rust-based paranormal research platform with:
//! - 50+ sensor types for multi-spectral anomaly detection
//! - Advanced entropy analysis (Shannon, Rényi, Tsallis, Sample, Permutation, etc.)
//! - Bayesian and Dempster-Shafer sensor fusion
//! - Military-grade AES-256-GCM encryption
//! - GPU-accelerated signal processing
//! - Native cross-platform GUI (Windows, macOS, Linux)
//!
//! # Architecture
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     GlowBarn Engine                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌──────────┐  ┌───────────┐  ┌────────────┐  │
//! │  │ Sensors │→ │ Analysis │→ │ Detection │→ │ Streaming  │  │
//! │  │ Manager │  │ Engine   │  │ Engine    │  │ Manager    │  │
//! │  └─────────┘  └──────────┘  └───────────┘  └────────────┘  │
//! │       ↓            ↓             ↓              ↓          │
//! │  ┌─────────────────────────────────────────────────────┐   │
//! │  │                    Event Bus                         │   │
//! │  └─────────────────────────────────────────────────────┘   │
//! │       ↓            ↓             ↓              ↓          │
//! │  ┌─────────┐  ┌──────────┐  ┌───────────┐  ┌────────────┐  │
//! │  │ Database│  │ Security │  │    GPU    │  │     UI     │  │
//! │  │         │  │ Manager  │  │  Compute  │  │  Console   │  │
//! │  └─────────┘  └──────────┘  └───────────┘  └────────────┘  │
//! └─────────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![allow(dead_code)]

pub mod core;
pub mod sensors;
pub mod analysis;
pub mod detection;
pub mod streaming;
pub mod security;
pub mod config;
pub mod db;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "gui")]
pub mod ui;

// Re-exports for convenience
pub use config::Config;
pub use core::{Engine, EventBus};
pub use sensors::{SensorManager, SensorReading, SensorType};
pub use analysis::AnalysisEngine;
pub use detection::{DetectionEngine, Detection, DetectionType};
pub use streaming::StreamingManager;
pub use security::SecurityManager;
pub use db::Database;

/// GlowBarn version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// GlowBarn name
pub const NAME: &str = "GlowBarn";

/// Build info
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: VERSION.to_string(),
        rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
        target: std::env::consts::ARCH.to_string(),
        os: std::env::consts::OS.to_string(),
        features: enabled_features(),
    }
}

/// Build information
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Version string
    pub version: String,
    /// Rust version
    pub rust_version: String,
    /// Target architecture
    pub target: String,
    /// Operating system
    pub os: String,
    /// Enabled features
    pub features: Vec<String>,
}

fn enabled_features() -> Vec<String> {
    let mut features = vec![];
    
    #[cfg(feature = "gui")]
    features.push("gui".to_string());
    
    #[cfg(feature = "gpu")]
    features.push("gpu".to_string());
    
    #[cfg(feature = "audio")]
    features.push("audio".to_string());
    
    #[cfg(feature = "serial")]
    features.push("serial".to_string());
    
    #[cfg(feature = "ml")]
    features.push("ml".to_string());
    
    features
}
