<?php

require_once __DIR__ . "/Chunked.php";
require_once __DIR__ . "/ResponseParser.php";

/**
 * Minimal TCP client for the mdis protocol. Uses only PHP's built-in
 * streams (fsockopen) — no external libraries, same spirit as the other
 * language clients.
 */
class MdisClient
{
    private string $host;
    private int $port;

    public static function connect(string $host = "127.0.0.1", int $port = 6411): self
    {
        return new self($host, $port);
    }

    private function __construct(string $host, int $port)
    {
        $this->host = $host;
        $this->port = $port;
    }

    /**
     * Stores $value under $key. $duration is the optional per-key
     * expiration in seconds; 0 (default) uses the server's EXPIRE_TIMEOUT.
     */
    public function set(string $key, string $value, int $duration = 0): string
    {
        $headers = "";
        if ($duration !== 0) {
            $headers .= "Duration: {$duration}\r\n";
        }

        $payload = $value;
        if (strlen($value) > Chunked::MAX_CHUNK_SIZE) {
            $headers .= "transfer-encoding: chunked\r\n";
            $payload = Chunked::encode($value);
        }

        return $this->sendCommand("set {$key}\r\n{$headers}\r\n{$payload}\r\n\r\n");
    }

    /** Retrieves the value stored under $key. */
    public function get(string $key): string
    {
        return $this->sendCommand("get {$key}\r\n");
    }

    private function sendCommand(string $message): string
    {
        $socket = fsockopen($this->host, $this->port, $errno, $errstr, 10);
        if ($socket === false) {
            throw new RuntimeException("Connection error: {$errstr} ({$errno})");
        }

        fwrite($socket, $message);
        stream_socket_shutdown($socket, STREAM_SHUT_WR);

        // The server closes the connection after replying.
        $response = stream_get_contents($socket);
        fclose($socket);

        return ResponseParser::parse($response === false ? "" : $response);
    }
}
