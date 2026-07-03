use std::{fmt, sync::Arc};
use log::debug;

use crate::storage::ShardStorage;

pub struct Engine {
    storage: ShardStorage
}

pub enum EngineOutput {
    StatusOk,
    Payload(Arc<[u8]>),
    NotFound,
}

impl Engine {
    pub fn new() -> Self {
        Self { storage: ShardStorage::new(100)}
    }

    pub async fn process(&self, command: &Arc<[u8]>) -> Result<EngineOutput, EngineRequestError> {
        let req = Request::parse(command);
        match req {
            // Added .await to async function execution paths
            Request::Set(SetRequest { key, value, ttl }) => self.set(key, value, ttl).await,
            Request::Get(GetRequest { key }) => self.get(key).await,
            Request::Invalid(InvalidRequest { reason }) => Err(EngineRequestError { details: reason }),
        }
    }

    pub async fn set(&self, key: &str, value: &str, ttl: u128) -> Result<EngineOutput, EngineRequestError> {
        debug!("Processing SET request: key={}, value={}", key, value);
        
        let k_arc: Arc<[u8]> = Arc::from(key.as_bytes());
        let v_arc: Arc<[u8]> = Arc::from(value.as_bytes());

        self.storage.set(k_arc, v_arc, ttl).await;
        return Ok(EngineOutput::StatusOk);
    }

    pub async fn get(&self, key: &str) -> Result<EngineOutput, EngineRequestError> {
        debug!("Processing GET request: {}", key);
        let k_arc: Arc<[u8]> = Arc::from(key.as_bytes());

        match self.storage.get(&k_arc).await {
            Some(val) => Ok(EngineOutput::Payload(val)),
            None => Ok(EngineOutput::NotFound),
        }
    }
}

#[derive(Debug)]
pub struct EngineRequestError {
    details: String,
}

impl fmt::Display for EngineRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Application Error: {}", self.details)
    }
}

impl std::error::Error for EngineRequestError {}

pub enum Request<'a> {
    Set(SetRequest<'a>),
    Get(GetRequest<'a>),
    Invalid(InvalidRequest)
}

impl<'a> Request<'a> {
    pub fn parse(req: &'a [u8]) -> Request<'a> {
        let raw_str = match std::str::from_utf8(req) {
            Ok(s) => s,
            Err(_) => return Request::Invalid(InvalidRequest { reason: "Invalid UTF-8".into() })
        };
        
        if raw_str.trim().is_empty() {
            return Request::Invalid(InvalidRequest {
                reason: String::from("Empty request"),
            });
        }

        let mut commands = raw_str.split_whitespace();
        let cmd = commands.next();

        match cmd {
            Some("GET") => {
                if let Some(key) = commands.next() {
                    Request::Get(GetRequest { key })
                } else {
                    Request::Invalid(InvalidRequest { reason: "Missing key for GET".into() })
                }
            }
            Some("SET") => {
                if let (Some(key), Some(val), Some(ttl_str)) = (commands.next(), commands.next(), commands.next()) {
                    match ttl_str.parse::<u128>() {
                        Ok(num) => Request::Set(SetRequest { key, value: val, ttl: num }),
                        Err(_) => Request::Invalid(InvalidRequest { reason: String::from("Invalid TTL") }),
                    }
                } else {
                    Request::Invalid(InvalidRequest { reason: "Missing arguments for SET".into() })
                }
            }
            _ => Request::Invalid(InvalidRequest { reason: String::from("Invalid command") })
        }
    }
}

pub struct SetRequest<'a> {
    pub key: &'a str,
    pub value: &'a str,
    pub ttl: u128
}

pub struct GetRequest<'a> {
    pub key: &'a str,
}

pub struct InvalidRequest {
    pub reason: String
}