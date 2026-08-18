# MDIS Client for Bash

A minimal Bash client for the mdis server. Uses only bash's built-in
`/dev/tcp` socket redirection — no `nc`/`socat`/external dependencies,
same spirit as the Python and Ruby clients.

## Requirements

- Bash built with `/dev/tcp` support (the default on Linux and macOS).
- `cat`, `printf` — both standard on any Unix-like system.

## Project Structure

```
examples/bash/simple/
├── src/
│   └── mdis_client.sh   # mdis_set / mdis_get functions
├── example_1.sh          # SET operation
├── example_2.sh          # GET operation
├── example_3.sh           # SET with per-key expiration
├── example_4.sh            # SET with a payload > 4096 bytes (chunked)
└── README.md
```

`src/mdis_client.sh` is meant to be sourced, not executed directly.

## Quick Start

```bash
source src/mdis_client.sh

# mdis_set host port key value [duration_sec]
resp=$(mdis_set "127.0.0.1" 6411 "token" "123456")

# mdis_get host port key
token=$(mdis_get "127.0.0.1" 6411 "token")
```

A fresh TCP connection is opened for every call — the server handles
one request per connection and closes it after replying, so there is
no persistent-connection state to manage.

## Running the examples

```bash
cd examples/bash/simple
./example_1.sh   # SET
./example_2.sh   # GET
./example_3.sh   # SET with a 2s expiration, GET after it lapses
./example_4.sh   # SET a 5000-byte payload (chunked transfer encoding)
```

## Protocol

See the main project's `AGENTS.md` for the full wire format. In short:

- `SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n`
- `GET {key}\r\n`
- Payloads over 4096 bytes must use chunked transfer encoding:
  `{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`.

## Response Handling

`mdis_client.sh` mirrors the response parsing used by the other
language clients:

- SET success: `OK\r\ninsert completed\r\n`
- GET found: `OK\r\n\r\n{data}\r\n\r\n`
- GET not found: `OK\r\n\r\n`
- Error / expired key: `Err\r\n`

`mdis_set`/`mdis_get` return just the parsed value (or `Error`/
`NO RESPONSE`), same as the Python and Ruby clients.
