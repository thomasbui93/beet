use log::{error};

mod server;
mod engine;
mod config;

fn main() {
    env_logger::init();
    let Ok(cfg) = config::Config::new(String::from(".env")) else {
        error!("Couldn't read .env file ");
        return;
    };
    let Some(server_cfg) = cfg.get_server_cfg() else {
        error!("Invalid server config");
        return;
    };
    let mut  server = server::Server::new(format!("{}:{}", server_cfg.host, server_cfg.port));
    server.start();
}
