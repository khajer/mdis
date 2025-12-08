const net = require("net");

class MdisClient {
  constructor(host = "127.0.0.1", port = 6411) {
    this.host = host;
    this.port = port;
  }

  set(key, value, expire_duration) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      client.connect(this.port, this.host, () => {
        let message;
        if (expire_duration !== undefined) {
          message = `set ${key} ${expire_duration}\r\n${value}`;
        } else {
          message = `set ${key}\r\n${value}`;
        }

        client.write(message);
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
        client.write(`get ${key}`);
        client.on("data", (data) => {
          client.end();
          resolve(parseResponse(data));
        });

        client.on("error", (err) => {
          reject(err);
        });
      });
    });
  }

  static connect(host = "127.0.0.1", port = 6411) {
    return new MdisClient(host, port);
  }
}

function parseResponse(data) {
  const response = data.toString();
  const resp = response.split("\r\n");

  // Remove empty strings that might result from split
  const filteredResp = resp.filter((line) => line !== "");

  if (filteredResp[0] && filteredResp[0].toLowerCase() === "ok") {
    // For successful get operations, the value is at index 1
    // For set operations, the message is at index 1
    if (filteredResp.length > 1) {
      return filteredResp[1];
    }
    // Empty value for key that doesn't exist
    return "";
  } else if (filteredResp[0] && filteredResp[0].toLowerCase() === "err") {
    // Error case
    if (filteredResp.length > 1) {
      return "Error:" + filteredResp[1];
    }
    return "Error";
  }
  return "NO RESPONSE";
}

module.exports = MdisClient;
