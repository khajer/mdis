import java.util.Objects;

/**
 * Plain check for the package-private parseResponse/chunkEncode helpers —
 * no test framework, matches the other language examples. Same (default)
 * package as MdisClient, so package-private access works.
 */
public class MdisClientTest {
    public static void main(String[] args) {
        testParseResponse();
        testParseResponseChunked();
        testChunkRoundTrip();
        System.out.println("All tests passed.");
    }

    private static void testParseResponse() {
        check(MdisClient.parseResponse("OK\r\ninsert completed\r\n"), "insert completed");
        check(MdisClient.parseResponse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
        check(MdisClient.parseResponse("OK\r\n\r\n"), "");
        check(MdisClient.parseResponse("Err\r\n"), "Error");
        check(MdisClient.parseResponse("garbage"), "NO RESPONSE");
    }

    private static void testParseResponseChunked() {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 4096 + 100; i++) {
            sb.append('a');
        }
        String data = sb.toString();
        String response = "OK\r\ntransfer-encoding: chunked\r\n\r\n" + MdisClient.chunkEncode(data);

        check(MdisClient.parseResponse(response), data);
    }

    private static void testChunkRoundTrip() {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 4096 * 2 + 10; i++) {
            sb.append('a');
        }
        String data = sb.toString();

        String encoded = MdisClient.chunkEncode(data);

        // Decode using the same rules the server uses and check we get the
        // original bytes back.
        StringBuilder decoded = new StringBuilder();
        String rest = encoded;
        while (true) {
            int nl = rest.indexOf("\r\n");
            int size = Integer.parseInt(rest.substring(0, nl), 16);
            rest = rest.substring(nl + 2);
            if (size == 0) {
                break;
            }
            decoded.append(rest, 0, size);
            rest = rest.substring(size + 2); // skip chunk + trailing CRLF
        }
        check(decoded.toString(), data);
    }

    private static void check(String actual, String expected) {
        if (!Objects.equals(actual, expected)) {
            throw new AssertionError("expected <" + expected + "> but got <" + actual + ">");
        }
    }
}
