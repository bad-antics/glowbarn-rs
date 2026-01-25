//! Data export functionality

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::{info, warn};

use crate::sensors::SensorReading;
use crate::detection::Detection;
use super::ExportFormat;

/// Data exporter
pub struct DataExporter {
    path: PathBuf,
    format: ExportFormat,
    readings_file: Mutex<Option<BufWriter<File>>>,
    detections_file: Mutex<Option<BufWriter<File>>>,
    readings_count: Mutex<usize>,
    detections_count: Mutex<usize>,
}

impl DataExporter {
    pub fn new(path: &str, format: ExportFormat) -> Result<Self> {
        let path = PathBuf::from(path);
        
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&path)?;
        
        Ok(Self {
            path,
            format,
            readings_file: Mutex::new(None),
            detections_file: Mutex::new(None),
            readings_count: Mutex::new(0),
            detections_count: Mutex::new(0),
        })
    }
    
    /// Export a sensor reading
    pub fn export_reading(&self, reading: &SensorReading) -> Result<()> {
        let mut file_lock = self.readings_file.lock().unwrap();
        
        // Open file if not already open
        if file_lock.is_none() {
            let filename = self.get_readings_filename();
            let file = self.open_export_file(&filename)?;
            *file_lock = Some(BufWriter::new(file));
            
            // Write header for CSV
            if self.format == ExportFormat::Csv {
                if let Some(ref mut writer) = *file_lock {
                    writeln!(writer, "timestamp,sensor_id,sensor_type,quality,data")?;
                }
            }
        }
        
        if let Some(ref mut writer) = *file_lock {
            match self.format {
                ExportFormat::Json => {
                    let json = serde_json::to_string(reading)?;
                    writeln!(writer, "{}", json)?;
                }
                ExportFormat::Csv => {
                    let data_str = reading.data.iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(";");
                    writeln!(writer, "{},{},{:?},{},{}", 
                        reading.timestamp.to_rfc3339(),
                        reading.sensor_id,
                        reading.sensor_type,
                        reading.quality,
                        data_str
                    )?;
                }
                ExportFormat::Binary => {
                    let bytes = bincode::serialize(reading)?;
                    let len = bytes.len() as u32;
                    writer.write_all(&len.to_le_bytes())?;
                    writer.write_all(&bytes)?;
                }
                ExportFormat::InfluxLineProtocol => {
                    let line = self.to_influx_line(reading);
                    writeln!(writer, "{}", line)?;
                }
            }
            
            writer.flush()?;
        }
        
        // Increment count
        let mut count = self.readings_count.lock().unwrap();
        *count += 1;
        
        // Rotate file every 100000 readings
        if *count % 100000 == 0 {
            drop(file_lock);
            self.rotate_readings_file()?;
        }
        
        Ok(())
    }
    
    /// Export a detection
    pub fn export_detection(&self, detection: &Detection) -> Result<()> {
        let mut file_lock = self.detections_file.lock().unwrap();
        
        if file_lock.is_none() {
            let filename = self.get_detections_filename();
            let file = self.open_export_file(&filename)?;
            *file_lock = Some(BufWriter::new(file));
            
            if self.format == ExportFormat::Csv {
                if let Some(ref mut writer) = *file_lock {
                    writeln!(writer, "timestamp,id,type,confidence,severity,sensor_count")?;
                }
            }
        }
        
        if let Some(ref mut writer) = *file_lock {
            match self.format {
                ExportFormat::Json => {
                    let json = serde_json::to_string(detection)?;
                    writeln!(writer, "{}", json)?;
                }
                ExportFormat::Csv => {
                    writeln!(writer, "{},{},{:?},{:.4},{:?},{}", 
                        detection.timestamp.to_rfc3339(),
                        detection.id,
                        detection.detection_type,
                        detection.confidence,
                        detection.severity,
                        detection.sensors.len()
                    )?;
                }
                ExportFormat::Binary => {
                    let bytes = bincode::serialize(detection)?;
                    let len = bytes.len() as u32;
                    writer.write_all(&len.to_le_bytes())?;
                    writer.write_all(&bytes)?;
                }
                ExportFormat::InfluxLineProtocol => {
                    let line = format!(
                        "detection,type={:?},severity={:?} confidence={},sensor_count={}i {}",
                        detection.detection_type,
                        detection.severity,
                        detection.confidence,
                        detection.sensors.len(),
                        detection.timestamp.timestamp_nanos_opt().unwrap_or(0)
                    );
                    writeln!(writer, "{}", line)?;
                }
            }
            
            writer.flush()?;
        }
        
        let mut count = self.detections_count.lock().unwrap();
        *count += 1;
        
        Ok(())
    }
    
    fn get_readings_filename(&self) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let ext = match self.format {
            ExportFormat::Json => "jsonl",
            ExportFormat::Csv => "csv",
            ExportFormat::Binary => "bin",
            ExportFormat::InfluxLineProtocol => "lp",
        };
        self.path.join(format!("readings_{}.{}", timestamp, ext))
    }
    
    fn get_detections_filename(&self) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let ext = match self.format {
            ExportFormat::Json => "jsonl",
            ExportFormat::Csv => "csv",
            ExportFormat::Binary => "bin",
            ExportFormat::InfluxLineProtocol => "lp",
        };
        self.path.join(format!("detections_{}.{}", timestamp, ext))
    }
    
    fn open_export_file(&self, path: &Path) -> Result<File> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .map_err(|e| anyhow!("Failed to open export file: {}", e))
    }
    
    fn rotate_readings_file(&self) -> Result<()> {
        let mut file_lock = self.readings_file.lock().unwrap();
        if let Some(mut writer) = file_lock.take() {
            writer.flush()?;
        }
        
        let filename = self.get_readings_filename();
        let file = self.open_export_file(&filename)?;
        *file_lock = Some(BufWriter::new(file));
        
        if self.format == ExportFormat::Csv {
            if let Some(ref mut writer) = *file_lock {
                writeln!(writer, "timestamp,sensor_id,sensor_type,quality,data")?;
            }
        }
        
        info!("Rotated readings export file to {:?}", filename);
        Ok(())
    }
    
    fn to_influx_line(&self, reading: &SensorReading) -> String {
        let mean = if reading.data.is_empty() {
            0.0
        } else {
            reading.data.iter().sum::<f64>() / reading.data.len() as f64
        };
        
        format!(
            "sensor,id={},type={:?} value={},quality={} {}",
            reading.sensor_id,
            reading.sensor_type,
            mean,
            reading.quality as i32,
            reading.timestamp.timestamp_nanos_opt().unwrap_or(0)
        )
    }
    
    /// Get export statistics
    pub fn get_stats(&self) -> (usize, usize) {
        let readings = *self.readings_count.lock().unwrap();
        let detections = *self.detections_count.lock().unwrap();
        (readings, detections)
    }
    
    /// Close all files
    pub fn close(&self) -> Result<()> {
        if let Some(mut writer) = self.readings_file.lock().unwrap().take() {
            writer.flush()?;
        }
        if let Some(mut writer) = self.detections_file.lock().unwrap().take() {
            writer.flush()?;
        }
        Ok(())
    }
}

/// Batch exporter for large datasets
pub struct BatchExporter {
    format: ExportFormat,
}

impl BatchExporter {
    pub fn new(format: ExportFormat) -> Self {
        Self { format }
    }
    
    /// Export readings to file
    pub fn export_readings<W: Write>(&self, readings: &[SensorReading], writer: &mut W) -> Result<()> {
        match self.format {
            ExportFormat::Json => {
                for reading in readings {
                    let json = serde_json::to_string(reading)?;
                    writeln!(writer, "{}", json)?;
                }
            }
            ExportFormat::Csv => {
                writeln!(writer, "timestamp,sensor_id,sensor_type,quality,mean_value")?;
                for reading in readings {
                    let mean = if reading.data.is_empty() {
                        0.0
                    } else {
                        reading.data.iter().sum::<f64>() / reading.data.len() as f64
                    };
                    writeln!(writer, "{},{},{:?},{},{:.6}", 
                        reading.timestamp.to_rfc3339(),
                        reading.sensor_id,
                        reading.sensor_type,
                        reading.quality,
                        mean
                    )?;
                }
            }
            ExportFormat::Binary => {
                for reading in readings {
                    let bytes = bincode::serialize(reading)?;
                    let len = bytes.len() as u32;
                    writer.write_all(&len.to_le_bytes())?;
                    writer.write_all(&bytes)?;
                }
            }
            ExportFormat::InfluxLineProtocol => {
                for reading in readings {
                    let mean = if reading.data.is_empty() {
                        0.0
                    } else {
                        reading.data.iter().sum::<f64>() / reading.data.len() as f64
                    };
                    writeln!(writer,
                        "sensor,id={},type={:?} value={},quality={} {}",
                        reading.sensor_id,
                        reading.sensor_type,
                        mean,
                        reading.quality as i32,
                        reading.timestamp.timestamp_nanos_opt().unwrap_or(0)
                    )?;
                }
            }
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Export detections to file
    pub fn export_detections<W: Write>(&self, detections: &[Detection], writer: &mut W) -> Result<()> {
        match self.format {
            ExportFormat::Json => {
                for detection in detections {
                    let json = serde_json::to_string(detection)?;
                    writeln!(writer, "{}", json)?;
                }
            }
            ExportFormat::Csv => {
                writeln!(writer, "timestamp,id,type,confidence,severity,sensor_count,correlation_score")?;
                for detection in detections {
                    writeln!(writer, "{},{},{:?},{:.4},{:?},{},{:.4}", 
                        detection.timestamp.to_rfc3339(),
                        detection.id,
                        detection.detection_type,
                        detection.confidence,
                        detection.severity,
                        detection.sensors.len(),
                        detection.correlation_score
                    )?;
                }
            }
            _ => {
                // Use JSON for other formats
                for detection in detections {
                    let json = serde_json::to_string(detection)?;
                    writeln!(writer, "{}", json)?;
                }
            }
        }
        
        writer.flush()?;
        Ok(())
    }
}
