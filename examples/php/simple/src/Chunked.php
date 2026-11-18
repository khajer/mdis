<?php

/**
 * Hex-size chunked transfer encoding used by the mdis server:
 * "{hex_size}\r\n{chunk}\r\n" repeated, terminated by "0\r\n\r\n".
 */
final class Chunked
{
    public const MAX_CHUNK_SIZE = 4096;
    public const RESPONSE_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

    public static function encode(string $data): string
    {
        $out = "";
        $length = strlen($data);
        for ($offset = 0; $offset < $length; $offset += self::MAX_CHUNK_SIZE) {
            $chunk = substr($data, $offset, self::MAX_CHUNK_SIZE);
            $out .= dechex(strlen($chunk)) . "\r\n" . $chunk . "\r\n";
        }

        return $out . "0\r\n\r\n";
    }

    /** Decodes a chunked body (everything after RESPONSE_PREFIX). */
    public static function decode(string $body): string
    {
        $out = "";
        $offset = 0;
        while (($nl = strpos($body, "\r\n", $offset)) !== false) {
            $size = hexdec(substr($body, $offset, $nl - $offset));
            if ($size === 0) {
                break;
            }

            $dataStart = $nl + 2;
            $out .= substr($body, $dataStart, $size);
            $offset = $dataStart + $size + 2; // skip the chunk's trailing \r\n
        }

        return $out;
    }
}
