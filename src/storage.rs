use std::{collections::HashMap, hash::{DefaultHasher, Hasher}, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use bytes::Bytes;
use log::debug;
use tokio::sync::RwLock;

use crate::config::StorageConfig;

pub struct StorageEntry {
    pub value: Bytes,
    pub ttl: u128,
}

pub struct Storage {
    pub kv: HashMap<Bytes, StorageEntry>,
}

impl Storage {
    pub fn new() -> Self {
        Self { kv: HashMap::new() }
    }

    pub fn start_eviction_loop(storage: Arc<RwLock<Self>>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            debug!("[Eviction] Background thread initialized and running!");

            loop {
                interval.tick().await;
                let mut lock = storage.write().await;
                if !lock.kv.is_empty() {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                    
                    lock.kv.retain(|key, entry| {
                        if entry.ttl < now {
                            if let Ok(s) = std::str::from_utf8(key) {
                                debug!("[Eviction] removing key {} now", s);
                            }
                            false
                        } else {
                            true
                        }
                    });
                }
            }
        });
    }

    pub fn set(&mut self, key: Bytes, value: Bytes, ttl: u128) -> Option<StorageEntry> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let ttl = now + ttl;
        self.kv.insert(key, StorageEntry { value, ttl })
    }

    pub fn get(&self, key: &Bytes) -> Option<Bytes> {
        match self.kv.get(key) {
            Some(entry) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                if entry.ttl < now { 
                    None 
                } else { 
                    Some(entry.value.clone())
                }
            }
            None => None,
        }
    }
}

pub struct ShardStorage {
    maps: Vec<Arc<RwLock<Storage>>>
}

impl ShardStorage {
    pub fn new(cfg: StorageConfig) -> Self {
        let mut maps = Vec::with_capacity(cfg.shard_count);
        for _ in 0..cfg.shard_count {
            let storage = Arc::new(RwLock::new(Storage::new()));
            maps.push(Arc::clone(&storage));
            Storage::start_eviction_loop(storage);
        }
        Self { maps }
    }

    pub async fn set(&self, key: Bytes, value: Bytes, ttl: u128) {
        let idx = self.hash(&key);
        let st = &self.maps[idx];
        let mut cache = st.write().await;
        cache.set(key, value, ttl);
    }

    pub async fn get(&self, key: &Bytes) -> Option<Bytes> {
        let idx = self.hash(key);
        let st = &self.maps[idx];
        let cache = st.read().await;
        cache.get(key)
    }

    // FIXED: Accept shared reference wrapper to avoid atomic mutations
    pub fn hash(&self, key: &Bytes) -> usize {
        let mut hasher_raw = DefaultHasher::new();
        hasher_raw.write(key.as_ref()); 
        
        let num_raw = hasher_raw.finish();
        (num_raw as usize) % self.maps.len()
    }
}