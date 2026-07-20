# Example 3: SET with a per-key expiration, then GET after it lapses.
require_relative "src/client"

client = MdisClient.connect("127.0.0.1", 6411)

resp = client.set("token", "123456", 2)
puts "resp: #{resp}"

sleep 3

token = client.get("token")
puts "token: #{token}"
