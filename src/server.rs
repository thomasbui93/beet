use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::engine::{Engine, EngineOutput};

pub struct Server {
    uri: String,
    engine: Arc<Engine>
}

impl Server {
    pub fn new(uri: String) -> Self {
        Self { 
            uri,
            engine: Arc::new(Engine::new())
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
            // 1. Read network data into our temporary chunk
            let bytes_read = match stream.read(&mut temp_chunk).await {
                Ok(0) => break, // Connection closed by client safely
                Ok(n) => n,
                Err(_) => break,
            };

            read_buffer.extend_from_slice(&temp_chunk[..bytes_read]);

            // 3. Scan the persistent buffer for our delimiter (e.g., newline `\n`)
            while let Some(delimiter_idx) = read_buffer.iter().position(|&b| b == b'\n') {
                
                // We found a complete message frame! Extract it up to the delimiter index
                let mut message_bytes = read_buffer.drain(..=delimiter_idx).collect::<Vec<u8>>();
                
                // Strip the trailing delimiter (\n or \r\n)
                if message_bytes.ends_with(b"\n") { message_bytes.pop(); }
                if message_bytes.ends_with(b"\r") { message_bytes.pop(); }

                if !message_bytes.is_empty() {
                    // 4. Freeze ONLY this complete message into an Arc for the Engine
                    let shared_msg: Arc<[u8]> = Arc::from(message_bytes);
                    
                    // Slice the Arc for keys/values safely as we did before
                    match engine.process(&shared_msg).await {
                        Ok(EngineOutput::Payload(value_bytes)) => {
                            stream.write_all(b"VALUE ").await;
                            stream.write_all(b" ").await;
                            stream.write_all(&value_bytes).await;
                            stream.write_all(b"\n").await;
                        }
                        Ok(EngineOutput::StatusOk) => {
                            stream.write_all(b"OK\n").await;
                        }
                        Ok(EngineOutput::NotFound) => {
                            stream.write_all(b"NOT_FOUND\n").await;
                        }
                        Err(_) => {
                            stream.write_all(b"ERROR\n").await;
                        }
                    }
                }
            }
        }
    }
}