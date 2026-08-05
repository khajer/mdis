# Example 1: SET a value.
require_relative "src/client"

client = MdisClient.connect("127.0.0.1", 6411)
resp = client.set("token", "123456")
puts "response: #{resp}"
