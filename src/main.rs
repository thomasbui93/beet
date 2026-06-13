mod server;
mod engine;

fn main() {
    env_logger::init();
    let server = server::Server::new(String::from("127.0.0.1:8080"));
    server.start();
}
