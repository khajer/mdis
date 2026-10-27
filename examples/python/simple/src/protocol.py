"""Wire-format helpers for the mdis protocol (no I/O)."""

MAX_BUFFER_SIZE = 4096
CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n"


def chunk_encode(data: bytes) -> bytes:
    """Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
    chunked transfer encoding: "{hex_size}\\r\\n{chunk}\\r\\n" repeated,
    terminated by "0\\r\\n\\r\\n"."""
    out = bytearray()
    for offset in range(0, len(data), MAX_BUFFER_SIZE):
        chunk = data[offset:offset + MAX_BUFFER_SIZE]
        out += f"{len(chunk):x}\r\n".encode("ascii") + chunk + b"\r\n"
    out += b"0\r\n\r\n"
    return bytes(out)


def chunk_decode(body: str) -> str:
    """Decodes a chunked response body back into the original value.
    Mirrors chunk_encode's framing. `body` is everything after
    CHUNKED_PREFIX."""
    out = []
    offset = 0
    while (nl := body.find("\r\n", offset)) != -1:
        size = int(body[offset:nl], 16)
        if size == 0:
            break
        data_start = nl + 2
        out.append(body[data_start:data_start + size])
        offset = data_start + size + 2  # skip the chunk's trailing \r\n
    return "".join(out)


def parse_response(data: str) -> str:
    """Parse a raw server response into a value.

      SET success:         OK\\r\\ninsert completed\\r\\n
      GET found:           OK\\r\\n\\r\\n{data}\\r\\n\\r\\n
      GET not found:       OK\\r\\n\\r\\n
      GET found (chunked): OK\\r\\ntransfer-encoding: chunked\\r\\n\\r\\n{chunks}
      error/expired key:   Err\\r\\n
    """
    # Chunk data may itself contain \r\n, so a chunked response must be
    # decoded from the raw string, not the \r\n-split list below.
    if data.startswith(CHUNKED_PREFIX):
        return chunk_decode(data[len(CHUNKED_PREFIX):])

    resp = data.split("\r\n")
    status = resp[0].lower()

    if status == "ok":
        if len(resp) >= 3 and resp[1] == "":
            return resp[2]  # GET with a value
        if len(resp) >= 2:
            return resp[1]  # SET message, or "" for GET not found
        return ""
    if status == "err":
        return "Error"
    return "NO RESPONSE"
