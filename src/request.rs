use core::fmt;

use bytes::Bytes;

#[derive(Debug)]
pub struct EngineRequestError {
    pub(crate) details: String,
}

impl fmt::Display for EngineRequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Application Error: {}", self.details)
    }
}

impl std::error::Error for EngineRequestError {}

pub enum Request {
    Set(SetRequest),
    Get(GetRequest),
    Invalid(InvalidRequest)
}

pub struct SetRequest {
    pub key: Bytes,   // Owned cheap reference-counted slice handles
    pub value: Bytes,
    pub ttl: u128,
}

pub struct GetRequest {
    pub key: Bytes,
}

pub struct InvalidRequest {
    pub reason: String,
}

impl Request {
    pub fn parse(req: &Bytes) -> Request {
        // 1. Get a zero-copy temporary string view to parse text locations
        let raw_str = match std::str::from_utf8(req) {
            Ok(s) => s,
            Err(_) => return Request::Invalid(InvalidRequest { reason: "Invalid UTF-8".into() })
        };
        
        if raw_str.trim().is_empty() {
            return Request::Invalid(InvalidRequest { reason: "Empty request".into() });
        }

        let mut commands = raw_str.split_whitespace();
        let cmd = commands.next();

        // Helper closure to cut a cheap sub-slice from the main `Bytes` allocation
        // based on where our `&str` iterator pointed.
        let slice_from_str = |sub_str: &str| -> Bytes {
            let start = sub_str.as_ptr() as usize - req.as_ptr() as usize;
            let end = start + sub_str.len();
            req.slice(start..end) // ZERO bytes copied, updates internal fat pointer
        };

        match cmd {
            Some("GET") => {
                if let Some(key_str) = commands.next() {
                    Request::Get(GetRequest { 
                        key: slice_from_str(key_str) 
                    })
                } else {
                    Request::Invalid(InvalidRequest { reason: "Missing key for GET".into() })
                }
            }
            Some("SET") => {
                if let (Some(key_str), Some(val_str), Some(ttl_str)) = (commands.next(), commands.next(), commands.next()) {
                    match ttl_str.parse::<u128>() {
                        Ok(num) => Request::Set(SetRequest { 
                            key: slice_from_str(key_str), 
                            value: slice_from_str(val_str), 
                            ttl: num 
                        }),
                        Err(_) => Request::Invalid(InvalidRequest { reason: "Invalid TTL".into() }),
                    }
                } else {
                    Request::Invalid(InvalidRequest { reason: "Missing arguments for SET".into() })
                }
            }
            _ => Request::Invalid(InvalidRequest { reason: "Invalid command".into() })
        }
    }
}