// Example 3: SET with a per-key expiration, then GET after it lapses.
package main

import (
	"fmt"
	"time"

	"mdis-client-example/client"
)

func main() {
	c := client.Connect("127.0.0.1", 6411)

	resp, err := c.Set("token", "123456", 2)
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println("resp:", resp)

	time.Sleep(3 * time.Second)

	token, err := c.Get("token")
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println("token:", token)
}
