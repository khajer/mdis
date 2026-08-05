# MDIS Client for C

A minimal C client for the mdis server. POSIX sockets only (`sys/socket.h`,
`arpa/inet.h`) — no external libraries, same spirit as the other language
clients.

## Project Structure

```
examples/c/simple/
├── src/
│   ├── mdis_client.h   # public API
│   └── mdis_client.c   # implementation
├── test/
│   └── test_client.c   # whitebox test (asserts, no framework)
├── example_1.c          # SET operation
├── example_2.c          # GET operation
├── example_3.c          # SET with per-key expiration
├── example_4.c          # SET with a payload > 4096 bytes (chunked)
├── Makefile
└── README.md
```

## Quick Start

```c
#include "src/mdis_client.h"

MdisClient client = mdis_connect("127.0.0.1", 6411);

char *resp = mdis_set(&client, "token", "123456", 0); // duration=0 -> server default TTL
char *token = mdis_get(&client, "token");

free(resp);
free(token);
```

`mdis_set`/`mdis_get` return a malloc'd string the caller must `free()`, or
`NULL` on connection error. `host` must be an IPv4 dotted-quad address (e.g.
`"127.0.0.1"`) — these examples don't need hostname resolution.

## Building and running

```bash
cd examples/c/simple
make               # builds example_1..example_4
./example_1        # SET
./example_2        # GET
./example_3        # SET with 2s expiration, GET after it lapses
./example_4        # SET a 5000-byte payload (chunked transfer)
```

## Testing

```bash
make test
```

Covers response parsing and the chunked-encoding round trip.

## API Reference

- `MdisClient mdis_connect(const char *host, int port)`
- `char *mdis_set(const MdisClient *client, const char *key, const char *value, long duration)` — `duration` in seconds, `0` for the server default
- `char *mdis_get(const MdisClient *client, const char *key)`

## Protocol

Same wire protocol as the other client examples — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).
