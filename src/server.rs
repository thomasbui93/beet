use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::config::StorageConfig;
use crate::engine::{Engine, EngineOutput};

pub struct Server {
    uri: String,
    engine: Arc<Engine>
}

impl Server {
    pub fn new(uri: String, storage_cfg: StorageConfig) -> Self {
        Self { 
            uri,
            engine: Arc::new(Engine::new(storage_cfg))
        }
    }

    pub async fn start(&mut self) {
        let listener = TcpListener::bind(&self.uri).await.unwrap();
        println!("Starting an async listener at {}", self.uri);

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let client_engine = Arc::clone(&self.engine);

            tokio::spawn(async move {
                Self::handle(client_engine, stream).await;
            });
        }
    }

    pub async fn handle(engine: Arc<Engine>, mut stream: TcpStream) {
        let mut read_buffer = Vec::with_capacity(4096);
        let mut temp_chunk = [0u8; 1024];

        loop {
            let bytes_read = match stream.read(&mut temp_chunk).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };

            read_buffer.extend_from_slice(&temp_chunk[..bytes_read]);
            while let Some(delimiter_idx) = read_buffer.iter().position(|&b| b == b'\n') {
                let mut message_bytes = read_buffer.drain(..=delimiter_idx).collect::<Vec<u8>>();

                if message_bytes.ends_with(b"\n") { message_bytes.pop(); }
                if message_bytes.ends_with(b"\r") { message_bytes.pop(); }

                if !message_bytes.is_empty() {
                    let shared_msg: Arc<[u8]> = Arc::from(message_bytes);

                    match engine.process(&shared_msg).await {
                        Ok(EngineOutput::Payload(value_bytes)) => {
                            let mut buf = Vec::with_capacity(7 + value_bytes.len());
                            buf.extend_from_slice(b"VALUE ");
                            buf.extend_from_slice(&value_bytes);
                            buf.extend_from_slice(b"\n");

                            let _ = stream.write_all(&buf).await;
                        }
                        Ok(EngineOutput::StatusOk) => {
                            let _ = stream.write_all(b"OK\n").await;
                        }
                        Ok(EngineOutput::NotFound) => {
                            let _ = stream.write_all(b"NOT_FOUND\n").await;
                        }
                        Err(_) => {
                            let _ = stream.write_all(b"ERROR\n").await;
                        }
                    }
                }
            }
        }
    }
}