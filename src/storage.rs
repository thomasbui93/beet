use std::{collections::HashMap, hash::{DefaultHasher, Hasher}, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use log::debug;
use tokio::sync::RwLock;

pub struct StorageEntry {
    pub value: Arc<[u8]>,
    pub ttl: u128,
}

pub struct Storage {
    pub kv: HashMap<Arc<[u8]>, StorageEntry>,
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
                    let sample_keys: Vec<Arc<[u8]>> = lock.kv.keys().cloned().collect();
                    
                    for key in sample_keys {
                        if let Some(entry) = lock.kv.get(&key) {
                            if entry.ttl < now {
                                debug!("[Eviction] removing key {} now", std::str::from_utf8(&key).unwrap());
                                lock.kv.remove(&key);
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn set(&mut self, key: Arc<[u8]>, value: Arc<[u8]>, ttl: u128) -> Option<StorageEntry> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let ttl = now + ttl;
        self.kv.insert(key, StorageEntry { value, ttl })
    }

    pub fn get(&self, key: &[u8]) -> Option<Arc<[u8]>> {
        match self.kv.get(key) {
            Some(entry) => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                if entry.ttl < now { 
                    None 
                } else { 
                    Some(Arc::clone(&entry.value)) 
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
    pub fn new(cap: usize) -> Self {
        let mut maps = Vec::with_capacity(cap);
        for _ in 0..cap {
            let storage = Arc::new(RwLock::new(Storage::new()));
            maps.push(Arc::clone(&storage));
            Storage::start_eviction_loop(storage);
        }
        Self { maps }
    }

    pub async fn set(&self, key: Arc<[u8]>, value: Arc<[u8]>, ttl: u128) {
        let idx = self.hash(&key);
        let st = &self.maps[idx];
        debug!("[write] key {} into hash {}", std::str::from_utf8(&key).unwrap(), idx);
        let mut cache = st.write().await;
        cache.set(key, value, ttl);
    }

    pub async fn get(&self, key: &[u8]) -> Option<Arc<[u8]>> {
        let idx = self.hash(key);
        let st = &self.maps[idx];
        debug!("[read] key {} from hash {}", std::str::from_utf8(&key).unwrap(), idx);
        let cache = st.read().await;
        cache.get(key)
    }

    pub fn hash(&self, key: &[u8]) -> usize {
        let mut hasher_raw = DefaultHasher::new();
        hasher_raw.write(key);
        let num_raw = hasher_raw.finish();
        (num_raw as usize) % self.maps.len()
    }
}