use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod shared;
use shared::ShareMemory;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*};

const DEFAULT_HOST: &str = "127.0.0.1:6411";

// ponytail: tiny .env parse, add dotenvy if more vars show up
fn host() -> String {
    std::env::var("HOST").ok().or_else(|| {
        std::fs::read_to_string(".env").ok()?
            .lines()
            .find_map(|l| l.strip_prefix("HOST=").map(|v| v.trim().to_string()))
    }).unwrap_or_else(|| DEFAULT_HOST.to_string())
}

// Logs go to both stdout and logs/<date>.log (one file per day; runs on the
// same day append to it). The returned guard must stay alive for the process
// lifetime or the file writer gets dropped.
fn setup_logging() -> WorkerGuard {
    std::fs::create_dir_all("logs").expect("failed to create logs directory");
    let filename = format!("{}.log", chrono::Local::now().format("%Y-%m-%d"));
    let file_appender = tracing_appender::rolling::never("logs", filename);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(fmt::layer().with_target(false))
        .with(
            fmt::layer()
                .with_target(false)
                .with_ansi(false)
                .with_writer(non_blocking),
        )
        .init();

    guard
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _log_guard = setup_logging();
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
