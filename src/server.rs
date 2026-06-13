use std::{io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}};
use log::{debug, error};

use crate::engine::Engine;

pub struct Server {
    uri: String,
    engine: Engine
}

impl Server {
    pub fn new(uri: String) -> Self {
        Self { 
            uri,
            engine: Engine::new()
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind(self.uri.clone()).unwrap();
        println!("Starting a listener at {}", self.uri);

        for stream in listener.incoming() {
            self.handle(stream.unwrap());
        } 
    }

    pub fn handle(&self, mut stream: TcpStream) {
        let buf_reader = BufReader::new(&stream);
        let command: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();
        debug!("Request: {command:#?}");
        for req in command.iter() {
            let response = self.engine.process(req.to_string());
            match response {
                Ok(res) => {
                    match res.to_string_json() {
                        Ok(js) => {
                            stream.write(js.as_bytes()).unwrap();
                        },
                        Err(err) => {
                            error!("Error: {err} on encoding");
                            stream.write_all("Encoding error".as_bytes()).unwrap()
                        }
                    }
                },
                Err(err) => {
                    error!("Error: {err} on request validation");
                    stream.write("Invalid request".as_bytes()).unwrap();
                } 
            }
        }
    }
}