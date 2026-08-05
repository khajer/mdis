// Example 1: SET a value.
public class Example1 {
    public static void main(String[] args) throws Exception {
        MdisClient client = MdisClient.connect("127.0.0.1", 6411);
        String resp = client.set("token", "123456");
        System.out.println("response: " + resp);
    }
}
