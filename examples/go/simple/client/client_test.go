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

func TestChunkEncodeRoundTrip(t *testing.T) {
	data := make([]byte, maxBufferSize*2+10)
	for i := range data {
		data[i] = 'a'
	}
	encoded := chunkEncode(string(data))

	// Re-decode using the same rules the server uses and check we get the
	// original bytes back.
	var decoded []byte
	rest := encoded
	for {
		nl := indexCRLF(rest)
		sizeHex := rest[:nl]
		size := 0
		for _, c := range []byte(sizeHex) {
			size = size*16 + hexDigit(c)
		}
		if size == 0 {
			break
		}
		rest = rest[nl+2:]
		decoded = append(decoded, rest[:size]...)
		rest = rest[size+2:] // skip chunk + trailing CRLF
	}
	if string(decoded) != string(data) {
		t.Fatalf("chunk round-trip mismatch: got %d bytes, want %d", len(decoded), len(data))
	}
}

func indexCRLF(s string) int {
	for i := 0; i+1 < len(s); i++ {
		if s[i] == '\r' && s[i+1] == '\n' {
			return i
		}
	}
	return -1
}

func hexDigit(c byte) int {
	switch {
	case c >= '0' && c <= '9':
		return int(c - '0')
	case c >= 'a' && c <= 'f':
		return int(c-'a') + 10
	default:
		return 0
	}
}
