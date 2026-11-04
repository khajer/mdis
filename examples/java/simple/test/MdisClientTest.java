import java.util.Objects;

/**
 * Plain check for the package-private ResponseParser/Chunked helpers —
 * no test framework, matches the other language examples. Same (default)
 * package as the client, so package-private access works.
 */
public class MdisClientTest {
    public static void main(String[] args) {
        testParseResponse();
        testParseResponseChunked();
        testChunkRoundTrip();
        System.out.println("All tests passed.");
    }

    private static void testParseResponse() {
        check(ResponseParser.parse("OK\r\ninsert completed\r\n"), "insert completed");
        check(ResponseParser.parse("OK\r\n\r\nvalue1\r\n\r\n"), "value1");
        check(ResponseParser.parse("OK\r\n\r\n"), "");
        check(ResponseParser.parse("Err\r\n"), "Error");
        check(ResponseParser.parse("garbage"), "NO RESPONSE");
    }

    private static void testParseResponseChunked() {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 4096 + 100; i++) {
            sb.append('a');
        }
        String data = sb.toString();
        String response = "OK\r\ntransfer-encoding: chunked\r\n\r\n" + Chunked.encode(data);

        check(ResponseParser.parse(response), data);
    }

    private static void testChunkRoundTrip() {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < 4096 * 2 + 10; i++) {
            sb.append('a');
        }
        String data = sb.toString();

        String encoded = Chunked.encode(data);

        String decoded = Chunked.decode(encoded);
        check(decoded, data);
    }

    private static void check(String actual, String expected) {
        if (!Objects.equals(actual, expected)) {
            throw new AssertionError("expected <" + expected + "> but got <" + actual + ">");
        }
    }
}
