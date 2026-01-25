// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Task scheduler for timed operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Interval;
use tracing::{debug, warn};

type TaskFn = Box<dyn Fn() + Send + Sync + 'static>;

struct ScheduledTask {
    name: String,
    interval: Duration,
    task: TaskFn,
    enabled: bool,
}

pub struct Scheduler {
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_task<F>(&self, name: &str, interval: Duration, task: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut tasks = self.tasks.write().await;
        tasks.insert(
            name.to_string(),
            ScheduledTask {
                name: name.to_string(),
                interval,
                task: Box::new(task),
                enabled: true,
            },
        );
        debug!("Scheduled task '{}' with interval {:?}", name, interval);
    }
    
    pub async fn remove_task(&self, name: &str) {
        let mut tasks = self.tasks.write().await;
        tasks.remove(name);
    }
    
    pub async fn enable_task(&self, name: &str, enabled: bool) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(name) {
            task.enabled = enabled;
        }
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
