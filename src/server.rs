use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use bytes::BytesMut;

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
        let mut read_buffer = BytesMut::with_capacity(4096);

        loop {
            let bytes_read = match stream.read_buf(&mut read_buffer).await {
                Ok(0) => break, // Connection closed
                Ok(n) => n,
                Err(_) => break,
            };

            while let Some(delimiter_idx) = read_buffer.iter().position(|&b| b == b'\n') {
                let mut line = read_buffer.split_to(delimiter_idx + 1);

                // Clean up the trailing delimiters in-place modifying just the length property
                if line.ends_with(b"\n") { line.truncate(line.len() - 1); }
                if line.ends_with(b"\r") { line.truncate(line.len() - 1); }

                if !line.is_empty() {
                    // Freeze turns the mutable view into an immutable, thread-safe, 
                    // atomic reference-counted 'Bytes' instance. 
                    // Total copy cost = 0 bytes.
                    let shared_msg = line.freeze(); 

                    // Pass the cheap atomic reference down into the engine
                    match engine.process(&shared_msg).await {
                        Ok(EngineOutput::Payload(value_bytes)) => {
                            if value_bytes.len() <= 2048 {
                                let mut stack_buf = [0u8; 2048 + 7];
                                stack_buf[0..6].copy_from_slice(b"VALUE ");
                                
                                let val_len = value_bytes.len();
                                stack_buf[6..6 + val_len].copy_from_slice(&value_bytes);
                                stack_buf[6 + val_len] = b'\n';
                                
                                // Exactly ONE system call, ONE await point, ZERO heap allocation
                                let _ = stream.write_all(&stack_buf[..7 + val_len]).await;
                            } else {
                                // Fallback for massive values (heap allocation is fine for rare anomalies)
                                let mut allocation = Vec::with_capacity(7 + value_bytes.len());
                                allocation.extend_from_slice(b"VALUE ");
                                allocation.extend_from_slice(&value_bytes);
                                allocation.extend_from_slice(b"\n");
                                let _ = stream.write_all(&allocation).await;
                            }
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