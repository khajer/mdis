// Example 2: GET a value.
package main

import (
	"fmt"

	"mdis-client-example/client"
)

func main() {
	c := client.Connect("127.0.0.1", 6411)
	token, err := c.Get("token")
	if err != nil {
		fmt.Println("Error:", err)
		return
	}
	fmt.Println("token:", token)
}
