// Example 4: SET a payload larger than 4096 bytes to exercise chunked
// transfer encoding.
public class Example4 {
    public static void main(String[] args) throws Exception {
        MdisClient client = MdisClient.connect("127.0.0.1", 6411);
        String txtData = "a".repeat(5000);

        String resp = client.set("token", txtData);
        System.out.println(resp);
    }
}
