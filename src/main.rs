use log::{error};

mod server;
mod engine;
mod config;
mod storage;
mod request;

#[tokio::main]
async fn main() {
    env_logger::init();
    let Ok(cfg) = config::Config::new(String::from(".env")) else {
        error!("Couldn't read .env file ");
        return;
    };
    let Some(server_cfg) = cfg.get_server_cfg() else {
        error!("Invalid server config");
        return;
    };

    let Some(storage_cfg) = cfg.get_storage_cfg() else {
        error!("Invalid storage config");
        return;
    };
    let mut server = server::Server::new(format!("{}:{}", server_cfg.host, server_cfg.port), storage_cfg);
    server.start().await;
}
