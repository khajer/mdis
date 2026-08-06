using System.Net.Sockets;
using System.Text;

[assembly: System.Runtime.CompilerServices.InternalsVisibleTo("MdisClient.Tests")]

namespace Mdis;

/// <summary>
/// Minimal TCP client for the mdis protocol. BCL only (System.Net.Sockets)
/// — no external packages, same spirit as the other language clients.
/// </summary>
public class MdisClient
{
    private const int MaxBufferSize = 4096;

    private readonly string _host;
    private readonly int _port;

    public static MdisClient Connect(string host, int port) => new(host, port);

    private MdisClient(string host, int port)
    {
        _host = host;
        _port = port;
    }

    /// <summary>
    /// Stores value under key. duration is the optional per-key expiration
    /// in seconds; 0 uses the server's EXPIRE_TIMEOUT.
    /// </summary>
    public string Set(string key, string value, long duration = 0)
    {
        var headers = new StringBuilder();
        if (duration != 0)
        {
            headers.Append($"Duration: {duration}\r\n");
        }

        var payload = value;
        if (value.Length > MaxBufferSize)
        {
            headers.Append("transfer-encoding: chunked\r\n");
            payload = ChunkEncode(value);
        }

        return Send($"set {key}\r\n{headers}\r\n{payload}\r\n\r\n");
    }

    /// <summary>Retrieves the value stored under key.</summary>
    public string Get(string key) => Send($"get {key}\r\n");

    private string Send(string message)
    {
        using var socket = new TcpClient(_host, _port);
        using var stream = socket.GetStream();

        var bytes = Encoding.UTF8.GetBytes(message);
        stream.Write(bytes, 0, bytes.Length);
        socket.Client.Shutdown(SocketShutdown.Send);

        using var buffer = new MemoryStream();
        stream.CopyTo(buffer); // server closes after replying
        return ParseResponse(Encoding.UTF8.GetString(buffer.ToArray()));
    }

    /// <summary>
    /// Splits data into MaxBufferSize chunks using the server's hex-size
    /// chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
    /// terminated by "0\r\n\r\n".
    /// </summary>
    internal static string ChunkEncode(string data)
    {
        var sb = new StringBuilder();
        var offset = 0;
        while (offset < data.Length)
        {
            var size = Math.Min(MaxBufferSize, data.Length - offset);
            sb.Append(Convert.ToString(size, 16)).Append("\r\n")
                .Append(data, offset, size).Append("\r\n");
            offset += size;
        }
        sb.Append("0\r\n\r\n");
        return sb.ToString();
    }

    /// <summary>
    /// Mirrors the Node.js/Python/Ruby/Go/Rust/C/Java client parsing of the
    /// same wire format:
    ///   SET success:       OK\r\ninsert completed\r\n
    ///   GET found:          OK\r\n\r\n{data}\r\n\r\n
    ///   GET not found:       OK\r\n\r\n
    ///   error/expired key:   Err\r\n
    /// </summary>
    internal static string ParseResponse(string data)
    {
        var parts = data.Split("\r\n");
        if (parts.Length == 0)
        {
            return "NO RESPONSE";
        }

        return parts[0].ToLowerInvariant() switch
        {
            "ok" when parts.Length >= 3 && parts[1].Length == 0 => parts[2], // GET with a value
            "ok" when parts.Length >= 2 && parts[1].Length != 0 => parts[1], // SET response message
            "ok" => "", // GET, key not found
            "err" => "Error",
            _ => "NO RESPONSE",
        };
    }
}
