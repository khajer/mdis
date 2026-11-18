const net = require("net");
const { buildSetMessage, parseResponse } = require("./protocol");

class MdisClient {
  constructor(host = "127.0.0.1", port = 6411) {
    this.host = host;
    this.port = port;
  }

  static connect(host = "127.0.0.1", port = 6411) {
    return new MdisClient(host, port);
  }

  set(key, dataInput, expireDuration) {
    return this._send(buildSetMessage(key, dataInput, expireDuration));
  }

  get(key) {
    return this._send(`get ${key}\r\n`);
  }

  // One short-lived connection per request. A response (especially a large
  // chunked one) can arrive across multiple "data" events -- buffer them all
  // and only parse once the server has closed the connection after replying.
  _send(message) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      const chunks = [];

      client.connect(this.port, this.host, () => client.write(message));
      client.on("data", (data) => chunks.push(data));
      client.on("end", () => resolve(parseResponse(Buffer.concat(chunks))));
      client.on("error", reject);
    });
  }
}

module.exports = MdisClient;
