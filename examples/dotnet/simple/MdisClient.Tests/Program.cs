// Plain check for the internal ParseResponse/ChunkEncode helpers — no test
// framework, matches the other language examples.
using Mdis;

TestParseResponse();
TestChunkRoundTrip();
Console.WriteLine("All tests passed.");

static void TestParseResponse()
{
    Check(MdisClient.ParseResponse("OK\r\ninsert completed\r\n"), "insert completed");
    Check(MdisClient.ParseResponse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    Check(MdisClient.ParseResponse("OK\r\n\r\n"), "");
    Check(MdisClient.ParseResponse("Err\r\n"), "Error");
    Check(MdisClient.ParseResponse("garbage"), "NO RESPONSE");
}

static void TestChunkRoundTrip()
{
    var data = new string('a', 4096 * 2 + 10);
    var encoded = MdisClient.ChunkEncode(data);

    // Decode using the same rules the server uses and check we get the
    // original bytes back.
    var decoded = new System.Text.StringBuilder();
    var rest = encoded;
    while (true)
    {
        var nl = rest.IndexOf("\r\n", StringComparison.Ordinal);
        var size = Convert.ToInt32(rest[..nl], 16);
        rest = rest[(nl + 2)..];
        if (size == 0)
        {
            break;
        }
        decoded.Append(rest[..size]);
        rest = rest[(size + 2)..]; // skip chunk + trailing CRLF
    }
    Check(decoded.ToString(), data);
}

static void Check(string actual, string expected)
{
    if (actual != expected)
    {
        throw new Exception($"expected <{expected}> but got <{actual}>");
    }
}
