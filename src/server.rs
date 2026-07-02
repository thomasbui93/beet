use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, sync::Arc};

use crate::engine::{Engine, EngineOutput};

pub struct Server {
    uri: String,
    engine: Engine,
    read_buffer: Vec<u8>,
}

impl Server {
    pub fn new(uri: String) -> Self {
        Self { 
            uri,
            engine: Engine::new(),
            read_buffer: Vec::with_capacity(4096)
        }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind(self.uri.clone()).unwrap();
        println!("Starting a listener at {}", self.uri);

        for stream in listener.incoming() {
            self.handle(stream.unwrap());
        } 
    }

    pub fn handle(&mut self, mut stream: TcpStream) {
        let mut temp_chunk = [0u8; 1024];

        loop {
            // 1. Read network data into our temporary chunk
            let bytes_read = match stream.read(&mut temp_chunk) {
                Ok(0) => break, // Connection closed by client
                Ok(n) => n,
                Err(_) => break,
            };

            self.read_buffer.extend_from_slice(&temp_chunk[..bytes_read]);

            // 3. Scan the persistent buffer for our delimiter (e.g., newline `\n`)
            while let Some(delimiter_idx) = self.read_buffer.iter().position(|&b| b == b'\n') {
                
                // We found a complete message frame! Extract it up to the delimiter index
                let mut message_bytes = self.read_buffer.drain(..=delimiter_idx).collect::<Vec<u8>>();
                
                // Strip the trailing delimiter (\n or \r\n)
                if message_bytes.ends_with(b"\n") { message_bytes.pop(); }
                if message_bytes.ends_with(b"\r") { message_bytes.pop(); }

                if !message_bytes.is_empty() {
                    // 4. Freeze ONLY this complete message into an Arc for the Engine
                    let shared_msg: Arc<[u8]> = Arc::from(message_bytes);
                    
                    // Slice the Arc for keys/values safely as we did before
                    match self.engine.process(&shared_msg) {
                        Ok(EngineOutput::Payload(value_bytes)) => {
                            stream.write_all(b"VALUE ").unwrap();
                            stream.write_all(b" ").unwrap();
                            stream.write_all(&value_bytes).unwrap();
                            stream.write_all(b"\n").unwrap();
                        }
                        Ok(EngineOutput::StatusOk) => {
                            stream.write_all(b"OK\n").unwrap();
                        }
                        Ok(EngineOutput::NotFound) => {
                            stream.write_all(b"NOT_FOUND\n").unwrap();
                        }
                        Err(_) => {
                            stream.write_all(b"ERROR\n").unwrap();
                        }
                    }
                }
            }
        }
    }
}