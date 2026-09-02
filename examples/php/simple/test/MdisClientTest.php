<?php

// Plain check for the parseResponse/chunkEncode helpers — no test
// framework, matches the other language examples.
require_once __DIR__ . "/../src/MdisClient.php";

function check(string $actual, string $expected): void
{
    if ($actual !== $expected) {
        throw new RuntimeException("expected <{$expected}> but got <{$actual}>");
    }
}

function test_parse_response(): void
{
    check(MdisClient::parseResponse("OK\r\ninsert completed\r\n"), "insert completed");
    check(MdisClient::parseResponse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    check(MdisClient::parseResponse("OK\r\n\r\n"), "");
    check(MdisClient::parseResponse("Err\r\n"), "Error");
    check(MdisClient::parseResponse("garbage"), "NO RESPONSE");
}

function test_chunk_round_trip(): void
{
    $data = str_repeat("a", 4096 * 2 + 10);
    $encoded = MdisClient::chunkEncode($data);

    // Decode using the same rules the server uses and check we get the
    // original bytes back.
    $decoded = "";
    $rest = $encoded;
    while (true) {
        $nl = strpos($rest, "\r\n");
        $size = hexdec(substr($rest, 0, $nl));
        $rest = substr($rest, $nl + 2);
        if ($size === 0) {
            break;
        }
        $decoded .= substr($rest, 0, $size);
        $rest = substr($rest, $size + 2); // skip chunk + trailing CRLF
    }
    check($decoded, $data);
}

test_parse_response();
test_chunk_round_trip();
echo "All tests passed.\n";
