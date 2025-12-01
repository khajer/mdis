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
    });
  }
  close() {
    this.client.destroy();
    console.log("Close connection");
  }
  set(key, value) {
    this.data[key] = value;
  }

  get(key) {
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
