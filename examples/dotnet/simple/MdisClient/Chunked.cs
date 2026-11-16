using System.Text;

namespace Mdis;

/// <summary>
/// Hex-size chunked transfer encoding used by the mdis server:
/// "{hex_size}\r\n{chunk}\r\n" repeated, terminated by "0\r\n\r\n".
/// </summary>
internal static class Chunked
{
    internal const int MaxChunkSize = 4096;
    internal const string ResponsePrefix = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

    internal static string Encode(string data)
    {
        var sb = new StringBuilder();
        for (var offset = 0; offset < data.Length; offset += MaxChunkSize)
        {
            var size = Math.Min(MaxChunkSize, data.Length - offset);
            sb.Append(Convert.ToString(size, 16)).Append("\r\n")
                .Append(data, offset, size).Append("\r\n");
        }
        return sb.Append("0\r\n\r\n").ToString();
    }

    /// <summary>Decodes a chunked body (everything after <see cref="ResponsePrefix"/>).</summary>
    internal static string Decode(string body)
    {
        var sb = new StringBuilder();
        var offset = 0;
        int nl;
        while ((nl = body.IndexOf("\r\n", offset, StringComparison.Ordinal)) != -1)
        {
            var size = Convert.ToInt32(body[offset..nl], 16);
            if (size == 0)
            {
                break;
            }

            var dataStart = nl + 2;
            sb.Append(body, dataStart, size);
            offset = dataStart + size + 2; // skip the chunk's trailing \r\n
        }
        return sb.ToString();
    }
}
