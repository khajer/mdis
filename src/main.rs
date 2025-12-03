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
        let shared_memory_clone = Arc::clone(&shared_memory);

        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);

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
                let resp;
                if let Ok(mut sm) = shared_memory_clone.lock() {
                    resp = sm.receive_message(incoming_message.to_string());
                } else {
                    resp = "".to_string();
                }

                // write to socket
                // println!("response: {}", resp);
                if let Err(e) = socket.write_all(resp.as_bytes()).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
