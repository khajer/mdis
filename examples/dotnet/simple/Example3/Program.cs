// Example 3: SET with a per-key expiration, then GET after it lapses.
using Mdis;

var client = MdisClient.Connect("127.0.0.1", 6411);

var resp = client.Set("token", "123456", 2);
Console.WriteLine($"resp: {resp}");

await Task.Delay(3000);

var token = client.Get("token");
Console.WriteLine($"token: {token}");
