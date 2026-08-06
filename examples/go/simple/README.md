# MDIS Client for Go

A minimal Go client for the mdis server. Stdlib only (`net`, `strings`) — no
external dependencies, same spirit as the Python client.

## Project Structure

```
examples/go/simple/
├── client/
│   └── client.go      # MdisClient implementation
├── example1/main.go   # SET operation
├── example2/main.go   # GET operation
├── example3/main.go   # SET with per-key expiration
├── example4/main.go   # SET with a payload > 4096 bytes (chunked)
├── go.mod
└── README.md
```

Each example lives in its own directory because Go allows only one `main`
per package/directory.

## Quick Start

```go
import "mdis-client-example/client"

c := client.Connect("127.0.0.1", 6411)

resp, err := c.Set("token", "123456", 0) // 0 = use server default TTL
token, err := c.Get("token")
```

## Running the examples

```bash
cd examples/go/simple
go run ./example1   # SET
go run ./example2   # GET
go run ./example3   # SET with 2s expiration, GET after it lapses
go run ./example4   # SET a 5000-byte payload (chunked transfer)
```

## API Reference

- `client.Connect(host string, port int) *MdisClient`
- `(*MdisClient) Set(key, value string, duration int) (string, error)` — `duration` in seconds, `0` for the server default
- `(*MdisClient) Get(key string) (string, error)`

## Protocol

Same wire protocol as the Node.js/Python clients — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).
