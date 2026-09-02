const net = require("net");

class MdisClient {
  constructor(host = "127.0.0.1", port = 6411) {
    this.host = host;
    this.port = port;
  }

  set(key, dataInput, expire_duration) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      const dataStr = dataInput.toString();
      let message;
      let value;

      client.connect(this.port, this.host, () => {
        let header_more = "";
        if (dataStr.length <= 4096) {
          value = dataInput;
        } else {
          header_more = "transfer-encoding: chunked\r\n";
          let chunkedData = "";
          let remainingData = dataStr;

          while (remainingData.length > 0) {
            const chunkSize = Math.min(4096, remainingData.length);
            const chunk = remainingData.substring(0, chunkSize);

            // Chunk sizes are hex per the server's protocol -- not decimal.
            chunkedData += `${chunkSize.toString(16)}\r\n${chunk}\r\n`;
            remainingData = remainingData.substring(chunkSize);
          }
          chunkedData += "0\r\n\r\n";
          value = chunkedData;
        }

        if (expire_duration !== undefined) {
          message = `set ${key}\r\n${header_more}duration: ${expire_duration}\r\n\r\n${value}\r\n\r\n`;
        } else {
          message = `set ${key}\r\n${header_more}\r\n${value}\r\n\r\n`;
        }

        client.write(message);
      });

      // A response (especially a large chunked one) can arrive across
      // multiple "data" events -- buffer them all and only parse once
      // the server has closed the connection after replying.
      const chunks = [];
      client.on("data", (data) => chunks.push(data));
      client.on("end", () => resolve(parseResponse(Buffer.concat(chunks))));
      client.on("error", (err) => reject(err));
    });
  }

  get(key) {
    return new Promise((resolve, reject) => {
      const client = new net.Socket();
      const chunks = [];

      client.connect(this.port, this.host, () => {
        client.write(`get ${key}\r\n`);
      });

      client.on("data", (data) => chunks.push(data));
      client.on("end", () => resolve(parseResponse(Buffer.concat(chunks))));
      client.on("error", (err) => reject(err));
    });
  }

  static connect(host = "127.0.0.1", port = 6411) {
    return new MdisClient(host, port);
  }
}

const CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

// Decodes a chunked response body back into the original value. Mirrors
// the chunk encoding above: "{hex_size}\r\n{chunk}\r\n" repeated,
// terminated by "0\r\n\r\n". `body` is everything after CHUNKED_PREFIX.
function chunkDecode(body) {
  let out = "";
  let offset = 0;
  for (;;) {
    const nl = body.indexOf("\r\n", offset);
    if (nl === -1) break;

    const size = parseInt(body.substring(offset, nl), 16);
    if (!size) break;

    const dataStart = nl + 2;
    out += body.substring(dataStart, dataStart + size);
    offset = dataStart + size + 2; // skip the chunk's trailing \r\n
  }
  return out;
}

function parseResponse(data) {
  const response = data.toString();

  // A large value comes back chunk-framed rather than as a plain
  // "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
  // \r\n, so it must be decoded from the raw string, not the \r\n-split
  // array used below for the other response shapes.
  if (response.startsWith(CHUNKED_PREFIX)) {
    return chunkDecode(response.slice(CHUNKED_PREFIX.length));
  }

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
