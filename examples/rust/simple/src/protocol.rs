//! Wire-format helpers for the mdis protocol (no I/O).

pub(crate) const MAX_BUFFER_SIZE: usize = 4096;

/// Splits `data` into `MAX_BUFFER_SIZE` chunks using the server's hex-size
/// chunked transfer encoding: `{hex_size}\r\n{chunk}\r\n` repeated,
/// terminated by `0\r\n\r\n`.
pub(crate) fn chunk_encode(data: &str) -> String {
    let mut out = String::new();
    for chunk in data.as_bytes().chunks(MAX_BUFFER_SIZE) {
        // Data is plain ASCII in these examples, so byte chunks are valid UTF-8.
        out.push_str(&format!(
            "{:x}\r\n{}\r\n",
            chunk.len(),
            std::str::from_utf8(chunk).unwrap()
        ));
    }
    out.push_str("0\r\n\r\n");
    out
}

pub(crate) const CHUNKED_PREFIX: &str = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

/// Decodes a chunked response body back into the original value. Mirrors
/// `chunk_encode`'s framing: `{hex_size}\r\n{chunk}\r\n` repeated,
/// terminated by `0\r\n\r\n`. `body` is everything after `CHUNKED_PREFIX`.
fn chunk_decode(body: &str) -> String {
    let mut out = String::new();
    let mut offset = 0;
    while let Some(nl) = body[offset..].find("\r\n") {
        let nl = offset + nl;
        let Ok(size) = usize::from_str_radix(&body[offset..nl], 16) else {
            break;
        };
        if size == 0 {
            break;
        }

        let data_start = nl + 2;
        out.push_str(&body[data_start..data_start + size]);
        offset = data_start + size + 2; // skip the chunk's trailing \r\n
    }
    out
}

/// Parses a raw server response:
///   SET success:        OK\r\ninsert completed\r\n
///   GET found:           OK\r\n\r\n{data}\r\n\r\n
///   GET not found:        OK\r\n\r\n
///   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
///   error/expired key:    Err\r\n
pub(crate) fn parse_response(data: &str) -> String {
    // A large value comes back chunk-framed rather than as a plain
    // "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
    // \r\n, so it must be decoded from the raw string, not the \r\n-split
    // vec used below for the other response shapes.
    if let Some(body) = data.strip_prefix(CHUNKED_PREFIX) {
        return chunk_decode(body);
    }

    let parts: Vec<&str> = data.split("\r\n").collect();

    match parts[0].to_lowercase().as_str() {
        "ok" => match parts[..] {
            [_, "", value, ..] => value.to_string(), // GET with a value
            [_, msg, ..] => msg.to_string(),         // SET message, or "" for GET not found
            _ => String::new(),
        },
        "err" => "Error".to_string(),
        _ => "NO RESPONSE".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_set_success() {
        assert_eq!(
            parse_response("OK\r\ninsert completed\r\n"),
            "insert completed"
        );
    }

    #[test]
    fn parses_get_found() {
        assert_eq!(parse_response("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    }

    #[test]
    fn parses_get_not_found() {
        assert_eq!(parse_response("OK\r\n\r\n"), "");
    }

    #[test]
    fn parses_error() {
        assert_eq!(parse_response("Err\r\n"), "Error");
    }

    #[test]
    fn parses_garbage() {
        assert_eq!(parse_response("garbage"), "NO RESPONSE");
    }

    #[test]
    fn parses_get_found_chunked() {
        let data = "a".repeat(MAX_BUFFER_SIZE + 100);
        let response = format!("{CHUNKED_PREFIX}{}", chunk_encode(&data));
        assert_eq!(parse_response(&response), data);
    }

    #[test]
    fn chunk_round_trip() {
        let data = "a".repeat(MAX_BUFFER_SIZE * 2 + 10);
        let encoded = chunk_encode(&data);

        assert_eq!(chunk_decode(&encoded), data);
    }
}
