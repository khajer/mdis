package client

import (
	"fmt"
	"strconv"
	"strings"
)

const (
	maxBufferSize = 4096
	chunkedPrefix = "OK\r\ntransfer-encoding: chunked\r\n\r\n"
)

// buildSetMessage builds a SET request. duration 0 uses the server's
// default EXPIRE_TIMEOUT.
func buildSetMessage(key, value string, duration int) string {
	var headers strings.Builder
	if duration != 0 {
		fmt.Fprintf(&headers, "Duration: %d\r\n", duration)
	}

	payload := value
	if len(value) > maxBufferSize {
		headers.WriteString("transfer-encoding: chunked\r\n")
		payload = chunkEncode(value)
	}

	return fmt.Sprintf("set %s\r\n%s\r\n%s\r\n\r\n", key, headers.String(), payload)
}

// chunkEncode splits data into maxBufferSize chunks using the server's
// hex-size chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
// terminated by "0\r\n\r\n".
func chunkEncode(data string) string {
	var out strings.Builder
	for len(data) > 0 {
		size := min(maxBufferSize, len(data))
		fmt.Fprintf(&out, "%x\r\n%s\r\n", size, data[:size])
		data = data[size:]
	}
	out.WriteString("0\r\n\r\n")
	return out.String()
}

// chunkDecode decodes a chunked response body (everything after
// chunkedPrefix) back into the original value.
func chunkDecode(body string) string {
	var out strings.Builder
	offset := 0
	for {
		nl := strings.Index(body[offset:], "\r\n")
		if nl == -1 {
			break
		}
		nl += offset

		n, err := strconv.ParseInt(body[offset:nl], 16, 64)
		if err != nil || n == 0 {
			break
		}
		size := int(n)

		dataStart := nl + 2
		out.WriteString(body[dataStart : dataStart+size])
		offset = dataStart + size + 2 // skip the chunk's trailing \r\n
	}
	return out.String()
}

// parseResponse extracts the value/message from a raw server response:
//
//	SET success:         OK\r\ninsert completed\r\n
//	GET found:           OK\r\n\r\n{data}\r\n\r\n
//	GET not found:       OK\r\n\r\n
//	GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
//	error/expired key:   Err\r\n
func parseResponse(data string) string {
	// Chunk data may itself contain \r\n, so a chunked response must be
	// decoded from the raw string, not the \r\n-split slice below.
	if strings.HasPrefix(data, chunkedPrefix) {
		return chunkDecode(data[len(chunkedPrefix):])
	}

	parts := strings.Split(data, "\r\n")

	switch strings.ToLower(parts[0]) {
	case "ok":
		switch {
		case len(parts) >= 3 && parts[1] == "":
			return parts[2] // GET with a value
		case len(parts) >= 2:
			return parts[1] // SET message, or "" for GET not found
		default:
			return ""
		}
	case "err":
		return "Error"
	default:
		return "NO RESPONSE"
	}
}
