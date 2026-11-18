<?php

require_once __DIR__ . "/Chunked.php";

/**
 * Parses server responses; mirrors the other language clients:
 *   SET success:         OK\r\ninsert completed\r\n
 *   GET found:           OK\r\n\r\n{data}\r\n\r\n
 *   GET not found:       OK\r\n\r\n
 *   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
 *   error/expired key:   Err\r\n
 */
final class ResponseParser
{
    public static function parse(string $data): string
    {
        // Chunk data may itself contain \r\n, so a chunked response must be
        // decoded from the raw string, not from the \r\n-split array below.
        if (strncmp($data, Chunked::RESPONSE_PREFIX, strlen(Chunked::RESPONSE_PREFIX)) === 0) {
            return Chunked::decode(substr($data, strlen(Chunked::RESPONSE_PREFIX)));
        }

        $parts = explode("\r\n", $data);

        switch (strtolower($parts[0])) {
            case "ok":
                if (count($parts) >= 3 && $parts[1] === "") {
                    return $parts[2]; // GET with a value
                }
                if (count($parts) >= 2 && $parts[1] !== "") {
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
