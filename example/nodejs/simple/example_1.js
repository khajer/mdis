const MdisClient = require("./src/index");

async function main() {
  try {
    let m = await MdisClient.connect((host = "127.0.0.1"), (port = 6411));

    m.set("token", "123456");
  } catch (error) {
    console.log(error);
  }
}
main();
