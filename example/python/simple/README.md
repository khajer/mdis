# MDIS Client for Python

A Python client library for connecting to the MDIS (Multi-Device Integration Service) server. This client provides a simple interface for storing and retrieving key-value pairs through TCP communication.

## Features

- TCP-based communication with the MDIS server
- Thread-safe operations
- Raw text-based protocol with delimiters
- Simple API with set/get operations
- Response parsing for server acknowledgments and errors

## Installation

No external dependencies are required. All functionality is implemented using Python's standard library.

## Usage

### Basic Connection

```python
from src import MdisClient

# Connect to the MDIS server
client = MdisClient.connect("127.0.0.1", 6411)

try:
    # Store a value
    client.set("token", "123456")
    
    # Retrieve a value
    token = client.get("token")
    print(f"Token: {token}")
    
finally:
    # Always close the connection
    client.close()
```

### Example 1: Setting a Value

See [example_1.py](example_1.py) for a complete example of setting a value:

```bash
python example_1.py
```

### Example 2: Getting a Value

See [example_2.py](example_2.py) for a complete example of retrieving a value:

```bash
python example_2.py
```

## API Reference

### MdisClient

#### Methods

- `__init__(host="localhost", port=6411)`: Create a new client instance
- `connect()`: Connect to the MDIS server
- `close()`: Close the connection to the server
- `set(key, value)`: Store a key-value pair
- `get(key)`: Retrieve a value by key

#### Static Methods

- `connect(host="localhost", port=6411)`: Connect to the server and return a connected client instance

## Protocol

The client communicates with the server using a simple text-based protocol:

1. SET commands use the format: `SET ${key}\n${value}\r\n`
2. GET commands use the format: `GET ${key}\r\n`
3. Responses are terminated with `\r\n`
4. No length prefix or JSON encoding is used

### Response Format

The server responds with one of the following formats:

1. Success response: `ok\nvalue\r\n`
2. Error response: `err\nerror message\r\n`

The client parses these responses and returns either the value (for successful operations) or an error message (for failed operations).

## Error Handling

The client handles errors in two ways:

1. **Connection/Protocol Errors**: The client raises exceptions for:
   - Connection errors
   - Protocol errors
   - Timeout errors (10 seconds)

2. **Server Errors**: The server may return error messages that are parsed by the client:
   - Server returns: `err\nerror message\r\n`
   - Client returns: `"Error:error message"`
   - If the response format is unrecognize

## Thread Safety

The client is thread-safe for basic operations. All socket operations are protected by a lock, and responses are matched to requests using unique IDs.
