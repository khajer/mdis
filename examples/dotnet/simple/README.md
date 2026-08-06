# MDIS Client for .NET

A minimal C# client for the mdis server. BCL only (`System.Net.Sockets`) —
no external NuGet packages, same spirit as the other language clients.

## Project Structure

```
examples/dotnet/simple/
├── MdisClient/                # class library
│   ├── MdisClient.cs
│   └── MdisClient.csproj
├── Example1/                  # SET operation
├── Example2/                  # GET operation
├── Example3/                  # SET with per-key expiration
├── Example4/                  # SET with a payload > 4096 bytes (chunked)
├── MdisClient.Tests/          # plain check, no test framework
├── MdisClientExamples.sln
└── README.md
```

Each example is its own console project referencing the `MdisClient`
library, mirroring the layout of the Go example.

## Quick Start

```csharp
using Mdis;

var client = MdisClient.Connect("127.0.0.1", 6411);

var resp = client.Set("token", "123456");   // server default TTL
var token = client.Get("token");
```

## Running the examples

```bash
cd examples/dotnet/simple
dotnet build
dotnet run --project Example1   # SET
dotnet run --project Example2   # GET
dotnet run --project Example3   # SET with 2s expiration, GET after it lapses
dotnet run --project Example4   # SET a 5000-byte payload (chunked transfer)
```

## Testing

```bash
dotnet run --project MdisClient.Tests
```

Covers response parsing and the chunked-encoding round trip.

## API Reference

- `MdisClient.Connect(string host, int port)`
- `client.Set(string key, string value, long duration = 0)` — `duration` in seconds, `0` for the server default
- `client.Get(string key)`

## Protocol

Same wire protocol as the other client examples — see the top-level
`AGENTS.md` for the full spec. In short:

```
SET {key}\r\n[Duration: {seconds}\r\n][transfer-encoding: chunked\r\n]\r\n{data}\r\n\r\n
GET {key}\r\n
```

Payloads over 4096 bytes are sent using hex-size chunked encoding
(`{hex_size}\r\n{chunk}\r\n` repeated, terminated by `0\r\n\r\n`).
