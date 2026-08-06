// Package client implements a minimal TCP client for the mdis protocol.
// Stdlib only (net, strings) — no external dependencies, matching the
// nodejs/python examples.
package client

import (
	"fmt"
	"net"
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

	buf := make([]byte, maxBufferSize)
	var resp strings.Builder
	for {
		n, err := conn.Read(buf)
		resp.Write(buf[:n])
		if err != nil {
			break // EOF or the server closed after replying
		}
		if n < maxBufferSize {
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

// parseResponse extracts the value/message from a raw server response.
// Mirrors the Node.js/Python client parsing of the same wire format:
//
//	SET success:      OK\r\ninsert completed\r\n
//	GET found:         OK\r\n\r\n{data}\r\n\r\n
//	GET not found:      OK\r\n\r\n
//	error/expired key:  Err\r\n
func parseResponse(data string) string {
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
