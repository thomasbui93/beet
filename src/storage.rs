use std::{collections::HashMap, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use log::{debug, info};
use tokio::sync::RwLock;

pub struct StorageEntry {
    pub value: Arc<[u8]>,
    pub ttl: u128,
}

pub struct Storage {
    // We make the underlying HashMap public or wrapped cleanly 
    // so you can use `.write().await` outside the struct.
    pub kv: HashMap<Arc<[u8]>, StorageEntry>,
}

impl Storage {
    // 1. Storage::new now returns the shared, locked instance directly
    pub fn new() -> Self {
        Self { kv: HashMap::new() }
    }

    pub fn start_eviction_loop(storage: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            info!("[Eviction] Background thread initialized and running!");

            loop {
                interval.tick().await;
                let mut lock = storage.write().await;
                if !lock.kv.is_empty() {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    let sample_keys: Vec<Arc<[u8]>> = lock.kv.keys().cloned().collect();
                    
                    for key in sample_keys {
                        if let Some(entry) = lock.kv.get(&key) {
                            if entry.ttl < now {
                                info!("[Eviction] removing key {} now", std::str::from_utf8(&key).unwrap());
                                lock.kv.remove(&key);
                            }
                        }
                    }
                }
            }
        });
    }

    // Standard synchronous-style methods inside (called AFTER you get the lock)
    pub fn set(&mut self, key: Arc<[u8]>, value: Arc<[u8]>, ttl: u128) -> Option<StorageEntry> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let ttl = now + ttl;
        self.kv.insert(key, StorageEntry { value, ttl })
    }

    pub fn get(&self, key: Arc<[u8]>) -> Option<&StorageEntry> {
        match self.kv.get(&key) {
            Some(entry) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                if entry.ttl < now { None } else { Some(entry) }
            }
            None => None,
        }
    }
}