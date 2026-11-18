/**
 * Hex-size chunked transfer encoding used by the mdis server:
 * "{hex_size}\r\n{chunk}\r\n" repeated, terminated by "0\r\n\r\n".
 */
final class Chunked {
    static final int MAX_CHUNK_SIZE = 4096;
    static final String RESPONSE_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

    private Chunked() {
    }

    static String encode(String data) {
        StringBuilder out = new StringBuilder();
        for (int offset = 0; offset < data.length(); offset += MAX_CHUNK_SIZE) {
            int end = Math.min(offset + MAX_CHUNK_SIZE, data.length());
            out.append(Integer.toHexString(end - offset)).append("\r\n")
                    .append(data, offset, end).append("\r\n");
        }
        return out.append("0\r\n\r\n").toString();
    }

    /** Decodes a chunked body (everything after {@link #RESPONSE_PREFIX}). */
    static String decode(String body) {
        StringBuilder out = new StringBuilder();
        int offset = 0;
        int nl;
        while ((nl = body.indexOf("\r\n", offset)) != -1) {
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
}
