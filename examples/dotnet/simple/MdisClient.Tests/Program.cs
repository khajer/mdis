// Plain check for the internal ResponseParser/Chunked helpers — no test
// framework, matches the other language examples.
using Mdis;

TestParseResponse();
TestParseResponseChunked();
TestChunkRoundTrip();
Console.WriteLine("All tests passed.");

static void TestParseResponse()
{
    Check(ResponseParser.Parse("OK\r\ninsert completed\r\n"), "insert completed");
    Check(ResponseParser.Parse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
    Check(ResponseParser.Parse("OK\r\n\r\n"), "");
    Check(ResponseParser.Parse("Err\r\n"), "Error");
    Check(ResponseParser.Parse("garbage"), "NO RESPONSE");
}

static void TestParseResponseChunked()
{
    var data = new string('a', 4096 + 100);
    var response = "OK\r\ntransfer-encoding: chunked\r\n\r\n" + Chunked.Encode(data);

    Check(ResponseParser.Parse(response), data);
}

static void TestChunkRoundTrip()
{
    var data = new string('a', 4096 * 2 + 10);
    var encoded = Chunked.Encode(data);

    var decoded = Chunked.Decode(encoded);
    Check(decoded, data);
}

static void Check(string actual, string expected)
{
    if (actual != expected)
    {
        throw new Exception($"expected <{expected}> but got <{actual}>");
    }
}
