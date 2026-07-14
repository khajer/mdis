const MdisClient = require("./src/index");

async function main() {
  try {
    let m = MdisClient.connect((host = "127.0.0.1"), (port = 6411));
    const resp = await m.set("token", "123456", 2);

    console.log("resp:", resp);

    setTimeout(async function () {
      const myToken = await m.get("token");

      console.log("token:", myToken);
    }, 1000 * 3);
    // console.log("completed");
  } catch (error) {
    console.log(error);
  }
}
main();
