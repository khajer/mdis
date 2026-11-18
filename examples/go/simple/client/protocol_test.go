package client

import "testing"

func TestParseResponse(t *testing.T) {
	cases := map[string]string{
		"OK\r\ninsert completed\r\n": "insert completed",
		"OK\r\n\r\nvalue1\r\n\r\n":   "value1",
		"OK\r\n\r\n":                 "",
		"Err\r\n":                    "Error",
		"garbage":                    "NO RESPONSE",
	}
	for in, want := range cases {
		if got := parseResponse(in); got != want {
			t.Errorf("parseResponse(%q) = %q, want %q", in, got, want)
		}
	}
}

func TestParseResponseChunked(t *testing.T) {
	data := make([]byte, maxBufferSize+100)
	for i := range data {
		data[i] = 'a'
	}
	want := string(data)
	response := "OK\r\ntransfer-encoding: chunked\r\n\r\n" + chunkEncode(want)

	if got := parseResponse(response); got != want {
		t.Fatalf("parseResponse(chunked) length = %d, want %d", len(got), len(want))
	}
}

func TestChunkEncodeRoundTrip(t *testing.T) {
	data := make([]byte, maxBufferSize*2+10)
	for i := range data {
		data[i] = 'a'
	}
	encoded := chunkEncode(string(data))

	decoded := chunkDecode(encoded)
	if decoded != string(data) {
		t.Fatalf("chunk round-trip mismatch: got %d bytes, want %d", len(decoded), len(data))
	}
}
