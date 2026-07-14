use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod shared;
use shared::ShareMemory;
use tracing::info;
use tracing_subscriber::fmt;

const DEFAULT_HOST: &str = "127.0.0.1:6411";

// ponytail: tiny .env parse, add dotenvy if more vars show up
fn host() -> String {
    std::env::var("HOST").ok().or_else(|| {
        std::fs::read_to_string(".env").ok()?
            .lines()
            .find_map(|l| l.strip_prefix("HOST=").map(|v| v.trim().to_string()))
    }).unwrap_or_else(|| DEFAULT_HOST.to_string())
}

fn setup_logging() {
    fmt()
        .with_target(false)
        .with_max_level(tracing::Level::INFO)
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging();
    let host = host();
    info!("Starting server at {host}");
    let listener = TcpListener::bind(&host).await?;
    let shared_memory = Arc::new(Mutex::new(ShareMemory::new()));

    loop {
        let shared_memory_clone = Arc::clone(&shared_memory);

        let (mut socket, addr) = listener.accept().await?;
        info!("Accepted connection from {addr}");

        tokio::spawn(async move {
            let mut sm = shared_memory_clone.lock().await;
            sm.socket_process(&mut socket).await;
        });
    }
}
