const net = require("net");

class MdisClient {
  constructor(host = "localhost", port = 6411) {
    this.host = host;
    this.port = port;
    this.data = {};
    this.client = new net.Socket();
  }
  connect() {
    return new Promise((resolve, reject) => {
      this.client.connect(this.port, this.host, () => {
        console.log(`Connected to server at ${this.host}:${this.port}`);
        resolve(this);
      });
      this.client.on("data", (data) => {
        // console.log(data.toString());
        this.client.end();
      });
      this.client.on("end", () => {
        console.log("disconnected from server");
      });
    });
  }
  close() {
    this.client.destroy();
    console.log("Close connection");
  }
  set(key, value) {
    this.data[key] = value;
    const data = `SET ${key}\n${value}\r\n`;
    this.client.write(data);
  }

  get(key) {
    const data = `GET ${key}\r\n`;
    this.client.write(data);
    return this.data[key];
  }
}

module.exports = {
  connect: async (localhost = "localhost", port = 6411) => {
    const client = new MdisClient(localhost, port);
    await client.connect();
    return client;
  },
};
