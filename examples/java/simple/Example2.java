// Example 2: GET a value.
public class Example2 {
    public static void main(String[] args) throws Exception {
        MdisClient client = MdisClient.connect("127.0.0.1", 6411);
        String token = client.get("token");
        System.out.println("token: " + token);
    }
}
