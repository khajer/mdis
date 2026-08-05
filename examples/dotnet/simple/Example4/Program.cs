// Example 4: SET a payload larger than 4096 bytes to exercise chunked
// transfer encoding.
using Mdis;

var client = MdisClient.Connect("127.0.0.1", 6411);
var txtData = new string('a', 5000);

var resp = client.Set("token", txtData);
Console.WriteLine(resp);
