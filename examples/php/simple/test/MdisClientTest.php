<?php

// Plain check for the ResponseParser/Chunked helpers — no test
// framework, matches the other language examples.
require_once __DIR__ . "/../src/ResponseParser.php";

function check(string $actual, string $expected): void
{
    if ($actual !== $expected) {
        throw new RuntimeException("expected <{$expected}> but got <{$actual}>");
    }
}

function test_parse_response(): void
{
    check(ResponseParser::parse("OK\r\ninsert completed\r\n"), "insert completed");
    check(ResponseParser::parse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    check(ResponseParser::parse("OK\r\n\r\n"), "");
    check(ResponseParser::parse("Err\r\n"), "Error");
    check(ResponseParser::parse("garbage"), "NO RESPONSE");
}

function test_parse_response_chunked(): void
{
    $data = str_repeat("a", 4096 + 100);
    $response = "OK\r\ntransfer-encoding: chunked\r\n\r\n" . Chunked::encode($data);

    check(ResponseParser::parse($response), $data);
}

function test_chunk_round_trip(): void
{
    $data = str_repeat("a", 4096 * 2 + 10);
    $encoded = Chunked::encode($data);

    $decoded = Chunked::decode($encoded);
    check($decoded, $data);
}

test_parse_response();
test_parse_response_chunked();
test_chunk_round_trip();
echo "All tests passed.\n";
