// Example 4: SET a payload larger than 4096 bytes to exercise chunked
// transfer encoding.
package main

import (
	"fmt"
	"strings"

	"mdis-client-example/client"
)

func main() {
	c := client.Connect("127.0.0.1", 6411)
	txtData := strings.Repeat("a", 5000)

	resp, err := c.Set("token", txtData, 0)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println(resp)
}
