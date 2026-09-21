# MDIS Client for PHP

A minimal PHP client for the mdis server. Uses only PHP's built-in streams
(`fsockopen`) — no external libraries, same spirit as the other language
clients.

## Project Structure

```
examples/php/simple/
├── src/
│   └── MdisClient.php    # client implementation
├── test/
│   └── MdisClientTest.php  # plain check, no test framework
├── example_1.php          # SET operation
├── example_2.php          # GET operation
├── example_3.php          # SET with per-key expiration
├── example_4.php          # SET with a payload > 4096 bytes (chunked)
├── composer.json
└── README.md
```

No Composer install is required to run the examples or tests — each file
`require_once`s `src/MdisClient.php` directly. `composer.json` is provided
only for PSR-4/classmap autoloading if you want to pull the client into a
larger project.

## Quick Start

```php
require_once "src/MdisClient.php";

$client = MdisClient::connect("127.0.0.1", 6411);

$resp = $client->set("token", "123456");  // server default TTL
$token = $client->get("token");
```

## Running the examples

```bash
cd examples/php/simple
php example_1.php   # SET
php example_2.php   # GET
php example_3.php   # SET with 2s expiration, GET after it lapses
php example_4.php   # SET a 5000-byte payload (chunked transfer)
```

## Testing

```bash
php test/MdisClientTest.php
```

Covers response parsing and the chunked-encoding round trip.

## API Reference

- `MdisClient::connect(string $host, int $port): MdisClient`
- `$client->set(string $key, string $value): string` — server default TTL
- `$client->set(string $key, string $value, int $duration): string` — `$duration` in seconds
- `$client->get(string $key): string`

## Protocol

Same wire protocol as the other client examples — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).

## Requirements

PHP 7.4 or later (uses typed properties and constant visibility modifiers).
No extensions beyond the default build are needed — `fsockopen` is part of
PHP core.
