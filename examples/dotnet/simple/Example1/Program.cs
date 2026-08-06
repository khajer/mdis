// Example 1: SET a value.
using Mdis;

var client = MdisClient.Connect("127.0.0.1", 6411);
var resp = client.Set("token", "123456");
Console.WriteLine($"response: {resp}");
