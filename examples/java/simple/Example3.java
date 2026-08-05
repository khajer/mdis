// Example 3: SET with a per-key expiration, then GET after it lapses.
public class Example3 {
    public static void main(String[] args) throws Exception {
        MdisClient client = MdisClient.connect("127.0.0.1", 6411);

        String resp = client.set("token", "123456", 2);
        System.out.println("resp: " + resp);

        Thread.sleep(3000);

        String token = client.get("token");
        System.out.println("token: " + token);
    }
}
