# MDIS Client for Python

A Python client library for connecting to the MDIS (Multi-Device Integration Service) server. This client provides a simple interface for storing and retrieving key-value pairs through TCP communication.

## Features

- TCP-based communication with the MDIS server
- Raw text-based protocol with delimiters
- Simple API with set/get operations
- Chunked transfer encoding for payloads over 4096 bytes
- Response parsing for server acknowledgments and errors

## Project Structure

```
example/python/simple/
├── src/
│   ├── __init__.py          # Module initialization
│   └── client.py            # MdisClient implementation
├── example_1.py             # Example showing SET operation
├── example_2.py             # Example showing GET operation
├── requirements.txt          # Python dependencies
├── setup.py                 # Package setup configuration
└── README.md               # This file
```

## Installation

No external dependencies are required for the basic client functionality. All functionality is implemented using Python's standard library.

To install the package:

```bash
pip install -e .
```

Or if you want to install it in development mode with optional dependencies:

```bash
pip install -e .[dev]
```

## Quick Start

### Basic Usage

```python
from src import MdisClient

# Connect to the MDIS server
client = MdisClient.connect("127.0.0.1", 6411)

try:
    # Store a value
    result = client.set("token", "123456")
    print(f"SET result: {result}")
    
    # Retrieve a value
    token = client.get("token")
    print(f"Token: {token}")
    
finally:
    # Always close the connection
    client.close()
```

## Examples

### Example 1: Setting a Value

This example demonstrates how to store a key-value pair:

```bash
python example_1.py
```

The example connects to the server and sets a token value. It demonstrates:
- Connection handling
- SET command format
- Response parsing
- Proper resource cleanup

### Example 2: Getting a Value

This example demonstrates how to retrieve a value by key:

```bash
python example_2.py
```

The example connects to the server and retrieves a token value. It demonstrates:
- Connection handling
- GET command format
- Response parsing
- Error handling

## API Reference

### MdisClient Class

#### Constructor

- `__init__(host="localhost", port=6411)`: Create a new client instance

#### Methods

- `connect()`: Connect to the MDIS server
- `close()`: Close the connection to the server
- `set(key, value)`: Store a key-value pair
  - Returns the value on success
  - Raises exception on connection error
- `get(key)`: Retrieve a value by key
  - Returns the value on success
  - Returns an error message string on server error
  - Raises exception on connection error

#### Static Methods

- `connect(host="localhost", port=6411)`: Connect to the server and return a connected client instance

## Protocol

The client communicates with the server using a simple text-based protocol.
Each `set()`/`get()` call opens its own short-lived connection -- the server
handles one request per connection and closes it after replying.

### Command Format

- SET: `set {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{value}\r\n\r\n`
- GET: `get {key}\r\n`
- Payloads over 4096 bytes must use chunked transfer encoding:
  `{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`

### Response Format

- SET success: `OK\r\ninsert completed\r\n`
- GET found: `OK\r\n\r\n{value}\r\n\r\n` (or chunk-framed for large values)
- GET not found: `OK\r\n\r\n`
- Error / expired key: `Err\r\n`

The client parses these responses and returns either the value (for
successful operations) or an error message (for failed operations).

### Example Protocol Exchange

```
Client: set token\r\n\r\n123456\r\n\r\n
Server: OK\r\ninsert completed\r\n

Client: get token\r\n
Server: OK\r\n\r\n123456\r\n\r\n

Client: get nonexistent\r\n
Server: OK\r\n\r\n
```

## Error Handling

The client handles errors in two ways:

1. **Connection/Protocol Errors**: The client raises exceptions for:
   - Connection errors
   - Timeout errors (10 seconds)

2. **Server Errors**: The server may return an error response:
   - Server returns: `Err\r\n` (e.g. for an expired key)
   - Client returns: `"Error"`
   - If the response format is unrecognizable, returns `"NO RESPONSE"`

## Connections

`set()`/`get()` are independent, self-contained calls -- each opens its own
socket, sends one request, reads the full response, and closes. There is no
persistent connection or background thread to manage; `connect()`/`close()`
are kept only so the existing call-site pattern (`connect()` ...
`finally: close()`) still works.

## Development

### Testing

For testing purposes, you can use the mock server implementation:

```python
from mock_server_example import MockMdisServer
from src import MdisClient

# Start a mock server
server = MockMdisServer()
server.start()

# Connect your client
client = MdisClient.connect("localhost", 6411)
# ... use client ...

# Clean up
client.close()
server.stop()
```

### Code Style

This project follows PEP 8 style guidelines. For development:

```bash
# Install development dependencies
pip install -e .[dev]

# Run formatter
black src/ *.py

# Run linter
flake8 src/ *.py

# Run tests
pytest
```

## Comparison with Node.js Version

This Python implementation provides the same functionality as the Node.js version with these differences:

1. **Protocol**: Both versions use the same raw TCP protocol
2. **API**: Similar API patterns with Pythonic naming conventions
3. **Connections**: Both open one connection per request, same as the Ruby/Go/Java clients
4. **Error Handling**: Python version uses exceptions for connection/timeout errors
5. **Dependencies**: Python version has no external dependencies

## License

This project is licensed under the MIT License.