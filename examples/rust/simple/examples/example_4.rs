//! Example 4: SET a payload larger than 4096 bytes to exercise chunked
//! transfer encoding.
use mdis_client_example::MdisClient;

fn main() {
    let client = MdisClient::connect("127.0.0.1", 6411);
    let txt_data = "a".repeat(5000);

    match client.set("token", &txt_data, 0) {
        Ok(resp) => println!("{resp}"),
        Err(e) => println!("Error: {e}"),
    }
}
