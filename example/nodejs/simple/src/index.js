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
          message = `set ${key}\r\nduration ${expire_duration}\r\n\r\n${value}\r\n`;
        } else {
          message = `set ${key}\r\n\r\n${value}\r\n`;
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
        client.write(`get ${key}\r\n`);
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

  // The first line should be OK or ERR
  const status = resp[0] ? resp[0].toLowerCase() : "";

  if (status === "ok") {
    // For get operations, format is "OK\r\n\r\n[value]\r\n"
    // For set operations, format is "OK\r\ninsert completed\r\n"
    if (resp.length >= 3 && resp[1] === "") {
      // Get operation with value
      return resp[2] || "";
    } else if (resp.length >= 2 && resp[1] !== "") {
      // Set operation response
      return resp[1];
    } else if (resp.length >= 2 && resp[1] === "") {
      // Get operation with empty value
      return "";
    }
    return "";
  } else if (status === "err") {
    // Error case
    return "Error";
  }
  return "NO RESPONSE";
}

module.exports = MdisClient;
