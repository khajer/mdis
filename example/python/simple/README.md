# MDIS Client for Python

A Python client library for connecting to the MDIS (Multi-Device Integration Service) server. This client provides a simple interface for storing and retrieving key-value pairs through TCP communication.

## Features

- TCP-based communication with the MDIS server
- Thread-safe operations
- Raw text-based protocol with delimiters
- Simple API with set/get operations
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

The client communicates with the server using a simple text-based protocol:

### Command Format

1. SET commands use the format: `SET ${key}\n${value}\r\n`
2. GET commands use the format: `GET ${key}\r\n`
3. Responses are terminated with `\r\n`
4. No length prefix or JSON encoding is used

### Response Format

The server responds with one of the following formats:

1. Success response: `ok\nvalue\r\n`
2. Error response: `err\nerror message\r\n`

The client parses these responses and returns either the value (for successful operations) or an error message (for failed operations).

### Example Protocol Exchange

```
Client: SET $token\n123456\r\n
Server: ok\n123456\r\n

Client: GET $token\r\n
Server: ok\n123456\r\n

Client: GET $nonexistent\r\n
Server: err\nKey not found\r\n
```

## Error Handling

The client handles errors in two ways:

1. **Connection/Protocol Errors**: The client raises exceptions for:
   - Connection errors
   - Protocol errors
   - Timeout errors (10 seconds)

2. **Server Errors**: The server may return error messages that are parsed by client:
   - Server returns: `err\nerror message\r\n`
   - Client returns: `"Error:error message"`
   - If the response format is unrecognizable, returns `"NO RESPONSE"`

## Thread Safety

The client is thread-safe for basic operations:
- All socket operations are protected by a lock
- Responses are matched to requests using unique IDs
- Connection state is properly synchronized

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
3. **Threading**: Python version uses threading for async operations
4. **Error Handling**: Python version uses exceptions for errors
5. **Dependencies**: Python version has no external dependencies

## License

This project is licensed under the MIT License.