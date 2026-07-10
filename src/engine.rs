use bytes::Bytes;

use crate::{config::StorageConfig, request::{EngineRequestError, Request, SetRequest, GetRequest, InvalidRequest}, storage::ShardStorage};

pub struct Engine {
    storage: ShardStorage
}

pub enum EngineOutput {
    StatusOk,
    Payload(Bytes),
    NotFound,
}

impl Engine {
    pub fn new(storage_cfg: StorageConfig) -> Self {
        Self { storage: ShardStorage::new(storage_cfg)}
    }

    pub async fn process(&self, command: &bytes::Bytes) -> Result<EngineOutput, EngineRequestError> {
        let req = Request::parse(command);
        match req {
            Request::Set(SetRequest { key, value, ttl }) => self.set(key, value, ttl).await,
            Request::Get(GetRequest { key }) => self.get(key).await,
            Request::Invalid(InvalidRequest { reason }) => Err(EngineRequestError { details: reason }),
        }
    }

    // Notice we take ownership of the `Bytes` tracking structures directly—zero copies!
    pub async fn set(&self, key: bytes::Bytes, value: bytes::Bytes, ttl: u128) -> Result<EngineOutput, EngineRequestError> {
        // We drop down into storage copy-free. The database just stores the pointers.
        self.storage.set(key, value, ttl).await;
        Ok(EngineOutput::StatusOk)
    }

    pub async fn get(&self, key: bytes::Bytes) -> Result<EngineOutput, EngineRequestError> {
        match self.storage.get(&key).await {
            Some(val) => Ok(EngineOutput::Payload(val)),
            None => Ok(EngineOutput::NotFound),
        }
    }
}