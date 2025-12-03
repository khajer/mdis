# import json
import socket
import threading
from typing import Any, Dict, Optional


class MdisClient:
    def __init__(self, host: str = "localhost", port: int = 6411):
        self.host = host
        self.port = port
        self.socket: Optional[socket.socket] = None
        self.connected = False
        self.data: Dict[str, Any] = {}
        self._lock = threading.Lock()
        self._response_handlers = {}
        self._request_id = 0
        self._receive_thread: Optional[threading.Thread] = None
        self._response_buffer = ""

    @staticmethod
    def connect(host: str = "localhost", port: int = 6411):
        """
        Connect to the MDIS server and return a client instance.

        Args:
            host: The server host
            port: The server port

        Returns:
            An instance of MdisClient connected to the server
        """
        client = MdisClient(host, port)
        client._connect()
        return client

    def _connect(self) -> None:
        """Connect to the MDIS server."""
        if self.connected:
            return

        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            self.connected = True
            # print(f"Connected to server at {self.host}:{self.port}")

            # Start receiving data in a separate thread
            self._receive_thread = threading.Thread(target=self._receive_data)
            self._receive_thread.daemon = True
            self._receive_thread.start()
        except Exception as e:
            print(f"Connection error: {e}")
            raise

    def close(self) -> None:
        """Close the connection to the server."""
        if self.socket:
            self.connected = False
            self.socket.close()
            if self._receive_thread and self._receive_thread.is_alive():
                self._receive_thread.join(timeout=1.0)
            print("Close connection")

    def _receive_data(self) -> None:
        """Receive data from the server in a separate thread."""
        buffer = ""

        while self.connected:
            try:
                data = self.socket.recv(4096).decode("utf-8")
                if not data:
                    break

                buffer += data

                # Process complete messages (terminated with \r\n)
                while "\r\n" in buffer:
                    line, buffer = buffer.split("\r\n", 1)
                    self._handle_response(line)

            except Exception as e:
                if self.connected:
                    print(f"Receive error: {e}")
                break

        self.connected = False

    def _handle_response(self, response: str) -> None:
        """Handle a response from the server."""
        # Store the last response for get operations
        self._response_buffer = response

        if self._request_id in self._response_handlers:
            handler = self._response_handlers[self._request_id]
            handler(response)
            del self._response_handlers[self._request_id]

    def _send_command(self, command: str) -> Any:
        """Send a command to the server and wait for the response."""
        if not self.connected:
            raise RuntimeError("Not connected to server")

        # Increment request ID for each command
        self._request_id += 1

        with self._lock:
            response = None
            response_received = threading.Event()

            def handle_response(resp):
                nonlocal response
                response = resp
                response_received.set()

            self._response_handlers[self._request_id] = handle_response
            self.socket.sendall(command.encode("utf-8"))

            # Wait for the response
            if not response_received.wait(timeout=10):
                del self._response_handlers[self._request_id]
                raise TimeoutError("Request timed out")

            return response

    def set(self, key: str, value: Any) -> Any:
        """Set a key-value pair."""
        with self._lock:
            self.data[key] = value

        command = f"SET {key}\n{value}\r\n"
        return self._send_command(command)

    def get(self, key: str) -> Any:
        """Get a value by key."""
        command = f"GET {key}\n\r\n"
        response = self._send_command(command)

        # Store the response locally for reference
        with self._lock:
            self.data[key] = response

        return response
