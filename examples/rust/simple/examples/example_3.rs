//! Example 3: SET with a per-key expiration, then GET after it lapses.
use mdis_client_example::MdisClient;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let client = MdisClient::connect("127.0.0.1", 6411);

    match client.set("token", "123456", 2) {
        Ok(resp) => println!("resp: {resp}"),
        Err(e) => {
            println!("Error: {e}");
            return;
        }
    }

    sleep(Duration::from_secs(3));

    match client.get("token") {
        Ok(token) => println!("token: {token}"),
        Err(e) => println!("Error: {e}"),
    }
}
