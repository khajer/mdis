//! Example 2: GET a value.
use mdis_client_example::MdisClient;

fn main() {
    let client = MdisClient::connect("127.0.0.1", 6411);
    match client.get("token") {
        Ok(token) => println!("token: {token}"),
        Err(e) => println!("Error: {e}"),
    }
}
