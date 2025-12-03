const MdisClient = require("./src/index");

async function main() {
  try {
    let m = MdisClient.connect((host = "127.0.0.1"), (port = 6411));
    const resp = await m.set("token", "123456");
    console.log(resp);
    // console.log("completed");
  } catch (error) {
    console.log(error);
  }
}
main();
