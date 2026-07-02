use std::{collections::HashMap, fmt, sync::Arc};
use log::debug;

pub struct Engine {
    kv: HashMap<Arc<[u8]>, Arc<[u8]>>
}

pub enum EngineOutput {
    /// Returned by SET operations on success
    StatusOk,
    /// Returned by GET when data exists (holds a zero-copy pointer)
    Payload(Arc<[u8]>),
    /// Returned by GET when a key is missing
    NotFound,
}

impl Engine {
    pub fn new() -> Self {
        Self { kv: HashMap::new() }
    }

    // We pass a reference to the Arc buffer for transient parsing
    pub fn process(&mut self, command: &Arc<[u8]>) -> Result<EngineOutput, EngineRequestError> {
        let req = Request::parse(command);
        match req {
            Request::Set(SetRequest { key, value, ttl }) => self.set(key, value, ttl),
            Request::Get(GetRequest { key }) => self.get(key),
            Request::Invalid(InvalidRequest { reason }) => Err(EngineRequestError { details: reason }),
        }
    }

    // Convert the zero-copy string slices into long-lived Arc allocations when inserting
    pub fn set(&mut self, key: &str, value: &str, _ttl: Option<u64>) -> Result<EngineOutput, EngineRequestError> {
        debug!("Processing SET request: key={}, value={}", key, value);
        
        let k_arc: Arc<[u8]> = Arc::from(key.as_bytes());
        let v_arc: Arc<[u8]> = Arc::from(value.as_bytes());
        
        self.kv.insert(k_arc, v_arc);
        return Ok(EngineOutput::StatusOk);
    }

    pub fn get(&self, key: &str) -> Result<EngineOutput, EngineRequestError> {
        debug!("Processing GET request: {}", key);
        
        // Query the map containing Arc<[u8]> using a byte slice view
        match self.kv.get(key.as_bytes()) {
            Some(value) => Ok(EngineOutput::Payload(Arc::clone(value))),
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
    // Borrow req with lifetime 'a to tie it to the returned Request tokens
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
                    match ttl_str.parse::<u64>() {
                        Ok(num) => Request::Set(SetRequest { key, value: val, ttl: Some(num) }),
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
    pub ttl: Option<u64>
}

pub struct GetRequest<'a> {
    pub key: &'a str,
}

pub struct InvalidRequest {
    pub reason: String
}