// Example 1: SET a value.
package main

import (
	"fmt"

	"mdis-client-example/client"
)

func main() {
	c := client.Connect("127.0.0.1", 6411)
	resp, err := c.Set("token", "123456", 0)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println("response:", resp)
}
