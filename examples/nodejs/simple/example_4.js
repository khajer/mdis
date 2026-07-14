const MdisClient = require("./src/index");

async function main() {
  try {
    let m = MdisClient.connect((host = "127.0.0.1"), (port = 6411));
    let txtData = "a".repeat(5000);
    const resp = await m.set("token", txtData);

    console.log(resp);
  } catch (error) {
    console.log(error);
  }
}
main();
