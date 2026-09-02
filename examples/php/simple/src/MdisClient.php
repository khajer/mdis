<?php

/**
 * Minimal TCP client for the mdis protocol. Uses only PHP's built-in
 * streams (fsockopen) — no external libraries, same spirit as the other
 * language clients.
 */
class MdisClient
{
    private const MAX_BUFFER_SIZE = 4096;

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
        if (strlen($value) > self::MAX_BUFFER_SIZE) {
            $headers .= "transfer-encoding: chunked\r\n";
            $payload = self::chunkEncode($value);
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

        $response = "";
        while (!feof($socket)) {
            $chunk = fread($socket, self::MAX_BUFFER_SIZE);
            if ($chunk === false) {
                break;
            }
            $response .= $chunk;
        }
        fclose($socket);

        return self::parseResponse($response); // server closes the connection after replying
    }

    /**
     * Splits $data into MAX_BUFFER_SIZE chunks using the server's hex-size
     * chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
     * terminated by "0\r\n\r\n".
     */
    public static function chunkEncode(string $data): string
    {
        $out = "";
        $offset = 0;
        $length = strlen($data);
        while ($offset < $length) {
            $size = min(self::MAX_BUFFER_SIZE, $length - $offset);
            $out .= dechex($size) . "\r\n" . substr($data, $offset, $size) . "\r\n";
            $offset += $size;
        }
        $out .= "0\r\n\r\n";

        return $out;
    }

    /**
     * Mirrors the Node.js/Python/Ruby/Java/Go/Rust/C client parsing of the
     * same wire format:
     *   SET success:       OK\r\ninsert completed\r\n
     *   GET found:         OK\r\n\r\n{data}\r\n\r\n
     *   GET not found:     OK\r\n\r\n
     *   error/expired key: Err\r\n
     */
    public static function parseResponse(string $data): string
    {
        $parts = explode("\r\n", $data);
        if (count($parts) === 0) {
            return "NO RESPONSE";
        }

        switch (strtolower($parts[0])) {
            case "ok":
                if (count($parts) >= 3 && $parts[1] === "") {
                    return $parts[2]; // GET with a value
                } elseif (count($parts) >= 2 && $parts[1] !== "") {
                    return $parts[1]; // SET response message
                }
                return ""; // GET, key not found
            case "err":
                return "Error";
            default:
                return "NO RESPONSE";
        }
    }
}
