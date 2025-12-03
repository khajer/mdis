const MdisClient = require("./src/index");

async function main() {
  try {
    let m = await MdisClient.connect((host = "127.0.0.1"), (port = 6411));

    const myToken = await m.get("token");
    console.log(myToken);
    console.log("completed");
    // m.close();
    //
  } catch (error) {
    console.log(error);
  }
}
main();
