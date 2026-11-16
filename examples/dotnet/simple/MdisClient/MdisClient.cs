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
        if (value.Length > Chunked.MaxChunkSize)
        {
            headers.Append("transfer-encoding: chunked\r\n");
            payload = Chunked.Encode(value);
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
        return ResponseParser.Parse(Encoding.UTF8.GetString(buffer.ToArray()));
    }
}
