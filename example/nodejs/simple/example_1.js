const MdisClient = require("./src/index");

async function main() {
  try {
    let m = await MdisClient.connect((host = "127.0.0.1"), (port = 6411));
    const resp = await m.set("token", "123456");
    console.log(resp);
    console.log("completed");
    // m.set(
    //   "jwt-token",
    //   "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTUxNjIzOTAyMn0.KMUFsIDTnFmyG3nMiGM6H9FNFUROf3wh7SmqJp-QV30",
    // );
  } catch (error) {
    console.log(error);
  }
}
main();
