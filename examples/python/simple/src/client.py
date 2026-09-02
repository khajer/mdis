import socket
from typing import Any

MAX_BUFFER_SIZE = 4096
CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n"


class MdisClient:
    """Minimal TCP client for the mdis protocol. Stdlib only (socket) --
    no external dependencies, same spirit as the Ruby/Node/Go clients.

    Each set()/get() call opens its own short-lived connection -- the
    server handles one request per connection and closes it after
    replying -- so there is no persistent-connection state to manage.
    """

    def __init__(self, host: str = "localhost", port: int = 6411):
        self.host = host
        self.port = port

    @staticmethod
    def connect(host: str = "localhost", port: int = 6411) -> "MdisClient":
        """Configure a client for host:port. Does not dial yet -- dialing
        happens per-request in set()/get()."""
        return MdisClient(host, port)

    def close(self) -> None:
        """No persistent connection is held, so there is nothing to
        close. Kept so existing call sites' finally-blocks still work."""
        pass

    def set(self, key: str, value: Any, duration: int = 0) -> str:
        """
        Store a key-value pair.

        Args:
            key: The key to store the value under
            value: The value to store
            duration: Optional per-key expiration in seconds; 0 (default)
                uses the server's EXPIRE_TIMEOUT

        Returns:
            The server's response value
        """
        value_bytes = str(value).encode("utf-8")

        headers = ""
        if duration != 0:
            headers += f"Duration: {duration}\r\n"

        if len(value_bytes) > MAX_BUFFER_SIZE:
            headers += "transfer-encoding: chunked\r\n"
            payload = _chunk_encode(value_bytes)
        else:
            payload = value_bytes

        message = f"set {key}\r\n{headers}\r\n".encode("utf-8") + payload + b"\r\n\r\n"
        return self._send_command(message)

    def get(self, key: str) -> str:
        """
        Retrieve a value by key.

        Args:
            key: The key to look up

        Returns:
            The stored value, "" if the key isn't found, or "Error" if
            the key has expired
        """
        return self._send_command(f"get {key}\r\n".encode("utf-8"))

    def _send_command(self, message: bytes) -> str:
        """Send a raw request and return the parsed response."""
        with socket.create_connection((self.host, self.port), timeout=10) as sock:
            sock.sendall(message)
            sock.shutdown(socket.SHUT_WR)

            chunks = []
            while True:
                data = sock.recv(MAX_BUFFER_SIZE)
                if not data:
                    break  # server closes the connection after replying
                chunks.append(data)

        response = b"".join(chunks).decode("utf-8", errors="replace")
        return parse_response(response)


def _chunk_encode(data: bytes) -> bytes:
    """Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
    chunked transfer encoding: "{hex_size}\\r\\n{chunk}\\r\\n" repeated,
    terminated by "0\\r\\n\\r\\n"."""
    out = bytearray()
    offset = 0
    while offset < len(data):
        size = min(MAX_BUFFER_SIZE, len(data) - offset)
        out += f"{size:x}\r\n".encode("ascii")
        out += data[offset:offset + size]
        out += b"\r\n"
        offset += size
    out += b"0\r\n\r\n"
    return bytes(out)


def _chunk_decode(body: str) -> str:
    """Decodes a chunked response body back into the original value.
    Mirrors _chunk_encode's framing. `body` is everything after
    CHUNKED_PREFIX."""
    out = []
    offset = 0
    while True:
        nl = body.find("\r\n", offset)
        if nl == -1:
            break

        size = int(body[offset:nl], 16)
        if size == 0:
            break

        data_start = nl + 2
        out.append(body[data_start:data_start + size])
        offset = data_start + size + 2  # skip the chunk's trailing \r\n

    return "".join(out)


def parse_response(data: str) -> str:
    """
    Mirrors the Ruby/Node.js client parsing of the same wire format:
      SET success:        OK\\r\\ninsert completed\\r\\n
      GET found:           OK\\r\\n\\r\\n{data}\\r\\n\\r\\n
      GET not found:        OK\\r\\n\\r\\n
      GET found (chunked): OK\\r\\ntransfer-encoding: chunked\\r\\n\\r\\n{chunks}
      error/expired key:    Err\\r\\n

    Args:
        data: The raw response data from the server

    Returns:
        The parsed response value or error message
    """
    # A large value comes back chunk-framed rather than as a plain
    # "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
    # \r\n, so it must be decoded from the raw string, not the \r\n-split
    # list used below for the other response shapes.
    if data.startswith(CHUNKED_PREFIX):
        return _chunk_decode(data[len(CHUNKED_PREFIX):])

    resp = data.split("\r\n")

    # The first line should be OK or ERR
    status = resp[0].lower() if resp else ""

    if status == "ok":
        # For get operations, format is "OK\r\n\r\n[value]\r\n"
        # For set operations, format is "OK\r\ninsert completed\r\n"
        if len(resp) >= 3 and resp[1] == "":
            # Get operation with value
            return resp[2] or ""
        elif len(resp) >= 2 and resp[1] != "":
            # Set operation response
            return resp[1]
        elif len(resp) >= 2 and resp[1] == "":
            # Get operation with empty value
            return ""
        return ""
    elif status == "err":
        # Error case
        return "Error"

    return "NO RESPONSE"
