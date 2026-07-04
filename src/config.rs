use std::{collections::HashMap, error::Error};
use std::fs::read_to_string;

pub struct Config {
    map: HashMap<String, String>
}

pub struct ServerConfig<'a> {
    pub port: &'a String,
    pub host: &'a String,
}

pub struct StorageConfig {
    pub shard_count: usize
}


impl Config {
    pub fn new(config_path: String) -> Result<Self, Box<dyn Error>> {
        let contents = read_to_string(config_path)?;
        let mut map = HashMap::new();
        
        for line in contents.lines() {
            if let Some((key, value)) = parse_line(line) {
                map.insert(key, value);
            } else {
                eprintln!("Skipping invalid line: {}", line);
            }
        }
        
        Ok(Self { map })
    }

    pub fn get(&self, key: &String) -> Option<&String> {
        return self.map.get(key) 
    }

    pub fn get_server_cfg<'a>(&'a self) -> Option<ServerConfig<'a>> {
        let port = self.get(&String::from("PORT"))?; 
        let host = self.get(&String::from("HOST"))?;
        Some(ServerConfig { port, host })
    }

    pub fn get_storage_cfg(&self) -> Option<StorageConfig> {
        match self.get(&String::from("SHARD_COUNT"))?.parse::<usize>() {
            Ok(val) => Some(StorageConfig { shard_count: val }),
            Err(_) => None
        }
    }
}

fn parse_line(line: &str) -> Option<(String, String)> {
    let parts = line.split('=').collect::<Vec<&str>>();
    if parts.len() == 2 {
        Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
    } else {
        None
    }
}