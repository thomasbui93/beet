use core::str;
use std::{error::Error, fmt};
use serde::Serialize;
use log::debug;

pub struct Engine {

}

impl Engine {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn process(&self, command: String) -> Result<Response, EngineRequestError> {
        let req = Request::parse(command);
        match req {
            Request::Set(SetRequest { key, value, ttl }) => self.set(key, value, ttl),
            Request::Get(GetRequest { key }) => self.get(key),
            Request::Invalid(InvalidRequest { reason }) => Err(EngineRequestError{details: reason}),
        }
    }

    pub fn set(&self, key: String, value: String, ttl: Option<u64>) -> Result<Response, EngineRequestError> {
        debug!("Processing SET request: {}{}{}", key, value, ttl.unwrap());
        Ok(Response::Set(SetResponse {}))
    }

    pub fn get(&self, key: String) -> Result<Response, EngineRequestError> {
        debug!("Processing GET request: {}", key);
        Ok(Response::Get(GetResponse {
            key: key,
            value: String::from("value"),
            ttl: 0
        }))
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

// 2. Implement the Error trait
impl std::error::Error for EngineRequestError {}

pub enum Request {
    Set(SetRequest),
    Get(GetRequest),
    Invalid(InvalidRequest)
}

impl Request {
    pub fn parse(req: String) -> Request {
        let commands: Vec<&str> = req.split_whitespace().collect();
        if commands.is_empty() {
            return Request::Invalid(InvalidRequest {
                reason: String::from("Empty request"),
            });
        }

        match commands[0] {
            "GET" => {
                if commands.len() != 2 {
                    Request::Invalid(InvalidRequest {
                        reason: String::from("GET request: Invalid arguments count."),
                    })
                } else {
                    Request::Get(GetRequest { key: commands[1].to_string() })
                }  
            },
            "SET" => {
                if commands.len() != 4 {
                    Request::Invalid(InvalidRequest {
                        reason: String::from("SET request: Invalid arguments count."),
                    })
                } else {
                    Request::Set(SetRequest {
                        key: commands[1].to_string(),
                        value: commands[2].to_string(),
                        ttl: commands.get(3).and_then(|t| t.parse::<u64>().ok()),
                    })
                }
            },
            _ => Request::Invalid(InvalidRequest {
                reason: String::from("Unsupported command")
            })
        }
    }
}

pub struct SetRequest {
    key: String,
    value: String,
    ttl: Option<u64>
}

pub struct GetRequest {
    key: String
}

pub struct InvalidRequest {
    reason: String
}

pub enum Response {
    Get(GetResponse),
    Set(SetResponse)
}

impl Response {
    pub fn to_string_json(&self) -> Result<String, Box<dyn Error>> {
        match self {
            Self::Get(req) => {
                let js = serde_json::to_string(req)?;
                Ok(js)
            }
            Self::Set(req) => {
                let js = serde_json::to_string(req)?;
                Ok(js)
            }
        }
    }
}

#[derive(Serialize, Debug)]
pub struct SetResponse {}

#[derive(Serialize, Debug)]
pub struct GetResponse {
    key: String,
    value: String,
    ttl: u64
}