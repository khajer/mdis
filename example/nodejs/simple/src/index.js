const net = require("net");

class MdisClient {
  constructor() {
    this.host = "127.0.0.1";
    this.port = 6411;
  }

  set(key, value) {
    return new Promise((resolve, reject) => {
      resolve("xx");
    });
  }

  get(key) {
    return new Promise((resolve, reject) => {
      resolve("yyy");
    });
  }

  static connect(host = "127.0.0.1", port = 6411) {
    const client = new MdisClient(host, port);
    return client;
  }
}

module.exports = MdisClient;
