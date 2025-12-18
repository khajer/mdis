use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

mod shared;
use shared::ShareMemory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "127.0.0.1:6411";
    println!("Starting server at {}", host);
    let listener = TcpListener::bind(host).await?;
    let shared_memory = Arc::new(Mutex::new(ShareMemory::new()));

    loop {
        let shared_memory_clone = Arc::clone(&shared_memory);

        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

        tokio::spawn(async move {
            let mut sm = shared_memory_clone.lock().await;
            sm.socket_process(&mut socket).await;
        });
    }
}
