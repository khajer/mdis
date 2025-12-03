const net = require("net");

class MdisClient {
  constructor() {
    this.host = "127.0.0.1";
    this.port = 6411;
  }

  set(key, value) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      client.connect(this.port, this.host, () => {
        client.write(`SET ${key}\n${value}\r\n`);
        client.on("data", (data) => {
          client.end();

          resolve(parseResponse(data));
        });
      });
    });
  }

  get(key) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      client.connect(this.port, this.host, () => {
        client.write(`GET ${key}\n\r\n`);
        client.on("data", (data) => {
          client.end();

          resolve(parseResponse(data));
        });
      });
    });
  }

  static connect(host = "127.0.0.1", port = 6411) {
    const client = new MdisClient(host, port);
    return client;
  }
}

function parseResponse(data) {
  let response = data.toString();

  let resp = response.split("\n");

  if (resp[0].toString().toLowerCase() === "ok") {
    return resp[1].toString().trim();
  } else if (resp[0].toString().toLowerCase() === "err") {
    return "Error:" + resp[1].toString().trim();
  }
  return "NO RESPONSE";
}

module.exports = MdisClient;
