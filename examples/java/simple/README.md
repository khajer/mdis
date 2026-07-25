# MDIS Client for Java

A minimal Java client for the mdis server. JDK only (`java.net.Socket`) —
no external libraries, same spirit as the other language clients.

## Project Structure

```
examples/java/simple/
├── src/
│   └── MdisClient.java   # client implementation
├── test/
│   └── MdisClientTest.java  # plain check, no test framework
├── Example1.java          # SET operation
├── Example2.java          # GET operation
├── Example3.java          # SET with per-key expiration
├── Example4.java          # SET with a payload > 4096 bytes (chunked)
├── Makefile
└── README.md
```

All classes live in the default (unnamed) package to keep things simple —
no `pom.xml`/`build.gradle` needed for four small examples.

## Quick Start

```java
MdisClient client = MdisClient.connect("127.0.0.1", 6411);

String resp = client.set("token", "123456");  // server default TTL
String token = client.get("token");
```

## Building and running

```bash
cd examples/java/simple
make                          # compiles everything into out/
java -cp out Example1         # SET
java -cp out Example2         # GET
java -cp out Example3         # SET with 2s expiration, GET after it lapses
java -cp out Example4         # SET a 5000-byte payload (chunked transfer)
```

## Testing

```bash
make test
```

Covers response parsing and the chunked-encoding round trip.

## API Reference

- `MdisClient.connect(String host, int port)`
- `client.set(String key, String value)` — server default TTL
- `client.set(String key, String value, long duration)` — `duration` in seconds
- `client.get(String key)`

## Protocol

Same wire protocol as the other client examples — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).
