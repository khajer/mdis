// Package client implements a minimal TCP client for the mdis protocol.
// Stdlib only (net, strings) — no external dependencies, matching the
// nodejs/python examples.
package client

import (
	"fmt"
	"net"
	"strings"
)

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
	return c.send(buildSetMessage(key, value, duration))
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
