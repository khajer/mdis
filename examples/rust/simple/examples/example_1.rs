//! Example 1: SET a value.
use mdis_client_example::MdisClient;

fn main() {
    let client = MdisClient::connect("127.0.0.1", 6411);
    match client.set("token", "123456", 0) {
        Ok(resp) => println!("response: {resp}"),
        Err(e) => println!("Error: {e}"),
    }
}
