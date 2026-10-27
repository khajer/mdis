import socket
from typing import Any

from .protocol import MAX_BUFFER_SIZE, chunk_encode, parse_response


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
            payload = chunk_encode(value_bytes)
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
