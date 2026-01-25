// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Database module for persistent storage

use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};

use crate::sensors::SensorReading;
use crate::detection::Detection;
use crate::config::DatabaseConfig;

/// Database manager
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    /// Open or create database
    pub fn open(config: &DatabaseConfig) -> Result<Self> {
        // Create parent directories
        if let Some(parent) = config.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&config.path)?;
        
        // Configure SQLite for performance
        conn.execute_batch(r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
        "#)?;
        
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            config: config.clone(),
        };
        
        db.create_tables()?;
        
        info!("Database opened at {:?}", config.path);
        Ok(db)
    }
    
    /// Create database tables
    fn create_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute_batch(r#"
            -- Sensor readings table
            CREATE TABLE IF NOT EXISTS readings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                sensor_id TEXT NOT NULL,
                sensor_type TEXT NOT NULL,
                quality REAL NOT NULL,
                data BLOB NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_readings_timestamp ON readings(timestamp);
            CREATE INDEX IF NOT EXISTS idx_readings_sensor ON readings(sensor_id);
            
            -- Detections table
            CREATE TABLE IF NOT EXISTS detections (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                detection_type TEXT NOT NULL,
                confidence REAL NOT NULL,
                severity TEXT NOT NULL,
                sensor_count INTEGER NOT NULL,
                entropy_deviation REAL,
                correlation_score REAL,
                classification TEXT,
                data BLOB NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE INDEX IF NOT EXISTS idx_detections_timestamp ON detections(timestamp);
            CREATE INDEX IF NOT EXISTS idx_detections_type ON detections(detection_type);
            
            -- Sessions table
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT,
                location TEXT,
                notes TEXT,
                reading_count INTEGER DEFAULT 0,
                detection_count INTEGER DEFAULT 0
            );
            
            -- Sensors table
            CREATE TABLE IF NOT EXISTS sensors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                sensor_type TEXT NOT NULL,
                calibration_data BLOB,
                last_seen TEXT,
                status TEXT DEFAULT 'unknown'
            );
            
            -- Settings table
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );
        "#)?;
        
        Ok(())
    }
    
    /// Store a sensor reading
    pub fn store_reading(&self, reading: &SensorReading) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let data = bincode::serialize(&reading.data)?;
        
        conn.execute(
            "INSERT INTO readings (timestamp, sensor_id, sensor_type, quality, data) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                reading.timestamp.to_rfc3339(),
                reading.sensor_id,
                format!("{:?}", reading.sensor_type),
                reading.quality,
                data
            ],
        )?;
        
        Ok(())
    }
    
    /// Store multiple readings in batch
    pub fn store_readings_batch(&self, readings: &[SensorReading]) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        let tx = conn.unchecked_transaction()?;
        let mut count = 0;
        
        for reading in readings {
            let data = bincode::serialize(&reading.data)?;
            
            tx.execute(
                "INSERT INTO readings (timestamp, sensor_id, sensor_type, quality, data) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    reading.timestamp.to_rfc3339(),
                    reading.sensor_id,
                    format!("{:?}", reading.sensor_type),
                    reading.quality,
                    data
                ],
            )?;
            count += 1;
        }
        
        tx.commit()?;
        Ok(count)
    }
    
    /// Store a detection
    pub fn store_detection(&self, detection: &Detection) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        let data = bincode::serialize(detection)?;
        let classification = detection.classification.as_ref()
            .map(|c| serde_json::to_string(c).ok())
            .flatten();
        
        conn.execute(
            r#"INSERT INTO detections 
               (id, timestamp, detection_type, confidence, severity, sensor_count, 
                entropy_deviation, correlation_score, classification, data)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            params![
                detection.id,
                detection.timestamp.to_rfc3339(),
                format!("{:?}", detection.detection_type),
                detection.confidence,
                format!("{:?}", detection.severity),
                detection.sensors.len() as i32,
                detection.entropy_deviation,
                detection.correlation_score,
                classification,
                data
            ],
        )?;
        
        Ok(())
    }
    
    /// Query readings by time range
    pub fn query_readings(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        sensor_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<StoredReading>> {
        let conn = self.conn.lock().unwrap();
        
        let sql = if let Some(sid) = sensor_id {
            format!(
                "SELECT id, timestamp, sensor_id, sensor_type, quality, data FROM readings 
                 WHERE timestamp >= ?1 AND timestamp <= ?2 AND sensor_id = ?3
                 ORDER BY timestamp DESC LIMIT {}",
                limit.unwrap_or(1000)
            )
        } else {
            format!(
                "SELECT id, timestamp, sensor_id, sensor_type, quality, data FROM readings 
                 WHERE timestamp >= ?1 AND timestamp <= ?2
                 ORDER BY timestamp DESC LIMIT {}",
                limit.unwrap_or(1000)
            )
        };
        
        let mut stmt = conn.prepare(&sql)?;
        
        let mut results = Vec::new();
        
        if let Some(sid) = sensor_id {
            let mut rows = stmt.query(params![start.to_rfc3339(), end.to_rfc3339(), sid])?;
            while let Some(row) = rows.next()? {
                results.push(StoredReading {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    sensor_id: row.get(2)?,
                    sensor_type: row.get(3)?,
                    quality: row.get(4)?,
                    data: row.get(5)?,
                });
            }
        } else {
            let mut rows = stmt.query(params![start.to_rfc3339(), end.to_rfc3339()])?;
            while let Some(row) = rows.next()? {
                results.push(StoredReading {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    sensor_id: row.get(2)?,
                    sensor_type: row.get(3)?,
                    quality: row.get(4)?,
                    data: row.get(5)?,
                });
            }
        }
        
        Ok(results)
    }
    
    /// Query detections by time range
    pub fn query_detections(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        min_confidence: Option<f64>,
        limit: Option<usize>,
    ) -> Result<Vec<StoredDetection>> {
        let conn = self.conn.lock().unwrap();
        
        let min_conf = min_confidence.unwrap_or(0.0);
        
        let sql = format!(
            "SELECT id, timestamp, detection_type, confidence, severity, sensor_count, data 
             FROM detections 
             WHERE timestamp >= ?1 AND timestamp <= ?2 AND confidence >= ?3
             ORDER BY timestamp DESC LIMIT {}",
            limit.unwrap_or(100)
        );
        
        let mut stmt = conn.prepare(&sql)?;
        
        let rows = stmt.query_map(params![start.to_rfc3339(), end.to_rfc3339(), min_conf], |row| {
            Ok(StoredDetection {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                detection_type: row.get(2)?,
                confidence: row.get(3)?,
                severity: row.get(4)?,
                sensor_count: row.get(5)?,
                data: row.get(6)?,
            })
        })?;
        
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        
        Ok(results)
    }
    
    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().unwrap();
        
        let reading_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM readings",
            [],
            |row| row.get(0),
        )?;
        
        let detection_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM detections",
            [],
            |row| row.get(0),
        )?;
        
        let size_bytes: i64 = conn.query_row(
            "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        ).unwrap_or(0);
        
        Ok(DatabaseStats {
            reading_count: reading_count as usize,
            detection_count: detection_count as usize,
            size_bytes: size_bytes as u64,
        })
    }
    
    /// Cleanup old data
    pub fn cleanup(&self, retention_days: u32) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        let deleted_readings = conn.execute(
            "DELETE FROM readings WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        
        let deleted_detections = conn.execute(
            "DELETE FROM detections WHERE timestamp < ?1",
            params![cutoff.to_rfc3339()],
        )?;
        
        // Vacuum to reclaim space
        conn.execute("VACUUM", [])?;
        
        info!("Cleaned up {} readings and {} detections older than {} days",
            deleted_readings, deleted_detections, retention_days);
        
        Ok(deleted_readings + deleted_detections)
    }
    
    /// Store a setting
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, Utc::now().to_rfc3339()],
        )?;
        
        Ok(())
    }
    
    /// Get a setting
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );
        
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredReading {
    pub id: i64,
    pub timestamp: String,
    pub sensor_id: String,
    pub sensor_type: String,
    pub quality: f32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct StoredDetection {
    pub id: String,
    pub timestamp: String,
    pub detection_type: String,
    pub confidence: f64,
    pub severity: String,
    pub sensor_count: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub reading_count: usize,
    pub detection_count: usize,
    pub size_bytes: u64,
}
