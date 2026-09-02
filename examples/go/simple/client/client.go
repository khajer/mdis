// Package client implements a minimal TCP client for the mdis protocol.
// Stdlib only (net, strings) — no external dependencies, matching the
// nodejs/python examples.
package client

import (
	"fmt"
	"net"
	"strconv"
	"strings"
)

const maxBufferSize = 4096

// MdisClient talks to an mdis server. Each call opens its own short-lived
// connection, same as the Node.js example client.
type MdisClient struct {
	addr string
}

// Connect returns a client configured for host:port. It does not dial yet;
// dialing happens per-request in Set/Get.
func Connect(host string, port int) *MdisClient {
	return &MdisClient{addr: fmt.Sprintf("%s:%d", host, port)}
}

// Set stores value under key. duration is the optional per-key expiration in
// seconds; pass 0 to use the server's default EXPIRE_TIMEOUT.
func (c *MdisClient) Set(key, value string, duration int) (string, error) {
	var headers strings.Builder
	if duration != 0 {
		fmt.Fprintf(&headers, "Duration: %d\r\n", duration)
	}

	payload := value
	if len(value) > maxBufferSize {
		headers.WriteString("transfer-encoding: chunked\r\n")
		payload = chunkEncode(value)
	}

	msg := fmt.Sprintf("set %s\r\n%s\r\n%s\r\n\r\n", key, headers.String(), payload)
	return c.send(msg)
}

// Get retrieves the value stored under key.
func (c *MdisClient) Get(key string) (string, error) {
	return c.send(fmt.Sprintf("get %s\r\n", key))
}

func (c *MdisClient) send(msg string) (string, error) {
	conn, err := net.Dial("tcp", c.addr)
	if err != nil {
		return "", err
	}
	defer conn.Close()

	if _, err := conn.Write([]byte(msg)); err != nil {
		return "", err
	}

	// Read until the server closes the connection after replying. A
	// single read can legitimately return fewer than maxBufferSize bytes
	// before the response is complete (e.g. mid-way through a large
	// chunked payload), so a short read is not itself end-of-stream --
	// only a read error (io.EOF once the peer closes) is.
	buf := make([]byte, maxBufferSize)
	var resp strings.Builder
	for {
		n, err := conn.Read(buf)
		if n > 0 {
			resp.Write(buf[:n])
		}
		if err != nil {
			break
		}
	}
	return parseResponse(resp.String()), nil
}

// chunkEncode splits data into maxBufferSize chunks using the server's
// hex-size chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
// terminated by "0\r\n\r\n".
func chunkEncode(data string) string {
	var out strings.Builder
	for len(data) > 0 {
		size := maxBufferSize
		if len(data) < size {
			size = len(data)
		}
		fmt.Fprintf(&out, "%x\r\n%s\r\n", size, data[:size])
		data = data[size:]
	}
	out.WriteString("0\r\n\r\n")
	return out.String()
}

const chunkedPrefix = "OK\r\ntransfer-encoding: chunked\r\n\r\n"

// chunkDecode decodes a chunked response body back into the original
// value. Mirrors chunkEncode's framing: "{hex_size}\r\n{chunk}\r\n"
// repeated, terminated by "0\r\n\r\n". body is everything after
// chunkedPrefix.
func chunkDecode(body string) string {
	var out strings.Builder
	offset := 0
	for {
		nl := strings.Index(body[offset:], "\r\n")
		if nl == -1 {
			break
		}
		nl += offset

		size, err := strconv.ParseInt(body[offset:nl], 16, 64)
		if err != nil || size == 0 {
			break
		}

		dataStart := nl + 2
		out.WriteString(body[dataStart : dataStart+int(size)])
		offset = dataStart + int(size) + 2 // skip the chunk's trailing \r\n
	}
	return out.String()
}

// parseResponse extracts the value/message from a raw server response.
// Mirrors the Node.js/Python client parsing of the same wire format:
//
//	SET success:       OK\r\ninsert completed\r\n
//	GET found:          OK\r\n\r\n{data}\r\n\r\n
//	GET not found:       OK\r\n\r\n
//	GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
//	error/expired key:   Err\r\n
func parseResponse(data string) string {
	// A large value comes back chunk-framed rather than as a plain
	// "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
	// \r\n, so it must be decoded from the raw string, not the \r\n-split
	// slice used below for the other response shapes.
	if strings.HasPrefix(data, chunkedPrefix) {
		return chunkDecode(data[len(chunkedPrefix):])
	}

	parts := strings.Split(data, "\r\n")
	if len(parts) == 0 {
		return "NO RESPONSE"
	}

	switch strings.ToLower(parts[0]) {
	case "ok":
		switch {
		case len(parts) >= 3 && parts[1] == "":
			return parts[2] // GET with a value
		case len(parts) >= 2 && parts[1] != "":
			return parts[1] // SET response message
		default:
			return "" // GET, key not found
		}
	case "err":
		return "Error"
	default:
		return "NO RESPONSE"
	}
}
