// Example 2: GET a value.
using Mdis;

var client = MdisClient.Connect("127.0.0.1", 6411);
var token = client.Get("token");
Console.WriteLine($"token: {token}");
