# Example 2: GET a value.
require_relative "src/client"

client = MdisClient.connect("127.0.0.1", 6411)
token = client.get("token")
puts "token: #{token}"
