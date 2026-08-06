# Example 4: SET a payload larger than 4096 bytes to exercise chunked
# transfer encoding.
require_relative "src/client"

client = MdisClient.connect("127.0.0.1", 6411)
txt_data = "a" * 5000

resp = client.set("token", txt_data)
puts resp
