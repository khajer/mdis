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
    private static final int MAX_BUFFER_SIZE = 4096;

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
        if (value.length() > MAX_BUFFER_SIZE) {
            headers.append("transfer-encoding: chunked\r\n");
            payload = chunkEncode(value);
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
            byte[] chunk = new byte[MAX_BUFFER_SIZE];
            int n;
            while ((n = in.read(chunk)) != -1) {
                buffer.write(chunk, 0, n);
            }
            return parseResponse(buffer.toString(StandardCharsets.UTF_8)); // server closes after replying
        }
    }

    /**
     * Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
     * chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
     * terminated by "0\r\n\r\n".
     */
    static String chunkEncode(String data) {
        StringBuilder out = new StringBuilder();
        int offset = 0;
        while (offset < data.length()) {
            int size = Math.min(MAX_BUFFER_SIZE, data.length() - offset);
            out.append(Integer.toHexString(size)).append("\r\n")
                    .append(data, offset, offset + size).append("\r\n");
            offset += size;
        }
        out.append("0\r\n\r\n");
        return out.toString();
    }

    private static final String CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

    /**
     * Decodes a chunked response body back into the original value. Mirrors
     * {@link #chunkEncode}'s framing: "{hex_size}\r\n{chunk}\r\n" repeated,
     * terminated by "0\r\n\r\n". {@code body} is everything after
     * {@link #CHUNKED_PREFIX}.
     */
    static String chunkDecode(String body) {
        StringBuilder out = new StringBuilder();
        int offset = 0;
        while (true) {
            int nl = body.indexOf("\r\n", offset);
            if (nl == -1) {
                break;
            }

            int size = Integer.parseInt(body.substring(offset, nl), 16);
            if (size == 0) {
                break;
            }

            int dataStart = nl + 2;
            out.append(body, dataStart, dataStart + size);
            offset = dataStart + size + 2; // skip the chunk's trailing \r\n
        }
        return out.toString();
    }

    /**
     * Mirrors the Node.js/Python/Ruby/Go/Rust/C client parsing of the same
     * wire format:
     * <pre>
     *   SET success:        OK\r\ninsert completed\r\n
     *   GET found:           OK\r\n\r\n{data}\r\n\r\n
     *   GET not found:        OK\r\n\r\n
     *   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
     *   error/expired key:    Err\r\n
     * </pre>
     */
    static String parseResponse(String data) {
        // A large value comes back chunk-framed rather than as a plain
        // "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
        // \r\n, so it must be decoded from the raw string, not the
        // \r\n-split array used below for the other response shapes.
        if (data.startsWith(CHUNKED_PREFIX)) {
            return chunkDecode(data.substring(CHUNKED_PREFIX.length()));
        }

        String[] parts = data.split("\r\n", -1);
        if (parts.length == 0) {
            return "NO RESPONSE";
        }

        switch (parts[0].toLowerCase()) {
            case "ok":
                if (parts.length >= 3 && parts[1].isEmpty()) {
                    return parts[2]; // GET with a value
                } else if (parts.length >= 2 && !parts[1].isEmpty()) {
                    return parts[1]; // SET response message
                }
                return ""; // GET, key not found
            case "err":
                return "Error";
            default:
                return "NO RESPONSE";
        }
    }
}
