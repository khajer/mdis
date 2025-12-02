use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

mod shared;
use shared::ShareMemory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:6411").await?;
    let shared_memory = Arc::new(Mutex::new(ShareMemory::new()));

    loop {
        let (mut socket, addr) = listener.accept().await?;

        println!("Accepted connection from {}", addr);
        let shared_memory_clone = Arc::clone(&shared_memory);
        tokio::spawn(async move {
            // buffer read
            let mut buf = [0; 4096];

            loop {
                // read from socket
                let n = match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };
                let incoming_message = String::from_utf8_lossy(&buf[..n]);
                println!("incomming : {}", incoming_message);
                //
                if let Ok(mut sm) = shared_memory_clone.lock() {
                    let resp = sm.receive_message(incoming_message.to_string());
                    println!("response : {}", resp);
                }

                // write to socket
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
