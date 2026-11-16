namespace Mdis;

/// <summary>
/// Parses server responses:
///   SET success:         OK\r\ninsert completed\r\n
///   GET found:           OK\r\n\r\n{data}\r\n\r\n
///   GET not found:       OK\r\n\r\n
///   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
///   error/expired key:   Err\r\n
/// </summary>
internal static class ResponseParser
{
    internal static string Parse(string data)
    {
        // Chunk data may itself contain \r\n, so a chunked response must be
        // decoded from the raw string, not the \r\n-split array below.
        if (data.StartsWith(Chunked.ResponsePrefix, StringComparison.Ordinal))
        {
            return Chunked.Decode(data[Chunked.ResponsePrefix.Length..]);
        }

        var parts = data.Split("\r\n");

        return parts[0].ToLowerInvariant() switch
        {
            "ok" when parts.Length >= 3 && parts[1].Length == 0 => parts[2], // GET with a value
            "ok" when parts.Length >= 2 => parts[1], // SET message, or "" for GET not found
            "ok" => "",
            "err" => "Error",
            _ => "NO RESPONSE",
        };
    }
}
