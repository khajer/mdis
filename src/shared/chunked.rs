use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use super::constants::{MAX_BUFFER_SIZE, NEW_LINE_BYTE, NEW_LINE_STR, TWO_DELIMITER};

/// Encode a value as a chunked response body (headers included).
pub fn encode_chunked(val: &str) -> String {
    let mut response = format!("OK{NEW_LINE_STR}transfer-encoding: chunked{TWO_DELIMITER}");
    let num_chunks = val.len() / MAX_BUFFER_SIZE;
    let remainder = val.len() % MAX_BUFFER_SIZE;

    for i in 0..num_chunks {
        let start = i * MAX_BUFFER_SIZE;
        push_chunk(&mut response, &val[start..start + MAX_BUFFER_SIZE]);
    }
    if remainder > 0 {
        push_chunk(&mut response, &val[num_chunks * MAX_BUFFER_SIZE..]);
    }

    response.push_str(&format!("0{TWO_DELIMITER}"));
    response
}

fn push_chunk(response: &mut String, chunk: &str) {
    response.push_str(&format!("{:x}{NEW_LINE_STR}", chunk.len()));
    response.push_str(chunk);
    response.push_str(NEW_LINE_STR);
}

/// Read a chunked request body from the socket. `remaining_data` holds bytes
/// already read past the header. Returns `None` on socket error.
pub async fn read_chunked_body(
    socket: &mut TcpStream,
    mut remaining_data: Vec<u8>,
) -> Option<Vec<u8>> {
    let mut buf = [0; MAX_BUFFER_SIZE];
    let mut complete_data = Vec::new();

    loop {
        let chunk_size_end = match remaining_data.windows(2).position(|w| w == NEW_LINE_BYTE) {
            Some(pos) => pos,
            None => match socket.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    remaining_data.extend_from_slice(&buf[..n]);
                    continue;
                }
                Err(_) => return None,
            },
        };

        let chunk_size_str = String::from_utf8_lossy(&remaining_data[..chunk_size_end]);
        let chunk_size = match usize::from_str_radix(chunk_size_str.trim(), 16) {
            Ok(size) => size,
            Err(_) => break,
        };
        if chunk_size == 0 {
            break;
        }

        let chunk_data_start = chunk_size_end + 2;
        let chunk_data_end = chunk_data_start + chunk_size;
        let trailing_crlf_end = chunk_data_end + 2;

        fill_to(socket, &mut remaining_data, chunk_data_end).await?;
        complete_data.extend_from_slice(&remaining_data[chunk_data_start..chunk_data_end]);
        fill_to(socket, &mut remaining_data, trailing_crlf_end).await?;

        remaining_data.drain(..trailing_crlf_end);
    }
    Some(complete_data)
}

/// Ensure `data` holds at least `target_len` bytes, reading the rest from the socket.
async fn fill_to(socket: &mut TcpStream, data: &mut Vec<u8>, target_len: usize) -> Option<()> {
    if data.len() < target_len {
        let mut extra = vec![0; target_len - data.len()];
        socket.read_exact(&mut extra).await.ok()?;
        data.extend_from_slice(&extra);
    }
    Some(())
}
