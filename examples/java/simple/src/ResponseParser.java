/**
 * Parses server responses:
 * <pre>
 *   SET success:         OK\r\ninsert completed\r\n
 *   GET found:           OK\r\n\r\n{data}\r\n\r\n
 *   GET not found:       OK\r\n\r\n
 *   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
 *   error/expired key:   Err\r\n
 * </pre>
 */
final class ResponseParser {
    private ResponseParser() {
    }

    static String parse(String data) {
        // Chunk data may itself contain \r\n, so a chunked response must be
        // decoded from the raw string, not the \r\n-split array below.
        if (data.startsWith(Chunked.RESPONSE_PREFIX)) {
            return Chunked.decode(data.substring(Chunked.RESPONSE_PREFIX.length()));
        }

        String[] parts = data.split("\r\n", -1);

        switch (parts[0].toLowerCase()) {
            case "ok":
                if (parts.length >= 3 && parts[1].isEmpty()) {
                    return parts[2]; // GET with a value
                }
                if (parts.length >= 2) {
                    return parts[1]; // SET message, or "" for GET not found
                }
                return "";
            case "err":
                return "Error";
            default:
                return "NO RESPONSE";
        }
    }
}
