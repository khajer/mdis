import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.Socket;
import java.nio.charset.StandardCharsets;

/**
 * Minimal TCP client for the mdis protocol. JDK only ({@code java.net.Socket})
 * — no external libraries, same spirit as the other language clients.
 */
public class MdisClient {
    private final String host;
    private final int port;

    public static MdisClient connect(String host, int port) {
        return new MdisClient(host, port);
    }

    private MdisClient(String host, int port) {
        this.host = host;
        this.port = port;
    }

    /** Stores value under key with the server's default expiration. */
    public String set(String key, String value) throws IOException {
        return set(key, value, 0);
    }

    /**
     * Stores value under key. duration is the optional per-key expiration
     * in seconds; 0 uses the server's EXPIRE_TIMEOUT.
     */
    public String set(String key, String value, long duration) throws IOException {
        StringBuilder headers = new StringBuilder();
        if (duration != 0) {
            headers.append("Duration: ").append(duration).append("\r\n");
        }

        String payload = value;
        if (value.length() > Chunked.MAX_CHUNK_SIZE) {
            headers.append("transfer-encoding: chunked\r\n");
            payload = Chunked.encode(value);
        }

        return send("set " + key + "\r\n" + headers + "\r\n" + payload + "\r\n\r\n");
    }

    /** Retrieves the value stored under key. */
    public String get(String key) throws IOException {
        return send("get " + key + "\r\n");
    }

    private String send(String message) throws IOException {
        try (Socket socket = new Socket(host, port)) {
            OutputStream out = socket.getOutputStream();
            out.write(message.getBytes(StandardCharsets.UTF_8));
            out.flush();
            socket.shutdownOutput();

            ByteArrayOutputStream buffer = new ByteArrayOutputStream();
            InputStream in = socket.getInputStream();
            byte[] chunk = new byte[Chunked.MAX_CHUNK_SIZE];
            int n;
            while ((n = in.read(chunk)) != -1) {
                buffer.write(chunk, 0, n);
            }
            return ResponseParser.parse(buffer.toString(StandardCharsets.UTF_8)); // server closes after replying
        }
    }
}
