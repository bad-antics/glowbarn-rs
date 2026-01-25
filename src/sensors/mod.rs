//! Sensor module - hardware interfaces and simulations

mod manager;
mod traits;
mod thermal;
mod seismic;
mod emf;
mod audio;
mod environmental;
mod radiation;
mod optical;
mod rf;
mod capacitive;
mod magnetic;
mod ionization;
mod quantum;
mod simulator;

pub use manager::SensorManager;
pub use traits::{Sensor, SensorReading, SensorType, SensorStatus, CalibrationData, SensorHealth};
pub use thermal::*;
pub use seismic::*;
pub use emf::*;
pub use audio::*;
pub use environmental::*;
pub use radiation::*;
pub use optical::*;
pub use rf::*;
pub use capacitive::*;
pub use magnetic::*;
pub use ionization::*;
pub use quantum::*;
pub use simulator::SensorSimulator;
