const net = require("net");

class MdisClient {
  constructor(host = "localhost", port = 6411) {
    this.host = host;
    this.port = port;
    this.data = {};
    this.client = new net.Socket();
    this.pendingRequests = new Map();
    this.requestId = 0;
  }
  connect() {
    return new Promise((resolve, reject) => {
      this.client.connect(this.port, this.host, () => {
        console.log(`Connected to server at ${this.host}:${this.port}`);
        resolve(this);
      });
      this.client.on("data", (data) => {
        const response = data.toString();

        // Parse response to extract request ID and actual response
        const lines = response.split("\n");
        if (lines.length >= 1) {
          const header = lines[0];
          const parts = header.split(" ");

          // For standard server responses without request ID, resolve all pending requests
          if (parts[0] === "Ok" || parts[0] === "Err") {
            // Resolve all pending requests with this response
            for (const [
              requestId,
              { resolve },
            ] of this.pendingRequests.entries()) {
              resolve(response);
              this.pendingRequests.delete(requestId);
            }
          }
        }
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
    return new Promise((resolve, reject) => {
      const requestId = (this.requestId++).toString();

      // Store the promise resolve/reject functions
      this.pendingRequests.set(requestId, { resolve, reject });

      // Set a timeout in case the server doesn't respond
      const timeout = setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error(`SET operation timed out for key: ${key}`));
        }
      }, 5000);

      // Override the resolve function to clear the timeout
      const originalResolve = resolve;
      resolve = (response) => {
        clearTimeout(timeout);
        originalResolve(response);
      };

      this.data[key] = value;
      const data = `SET ${key}\n${value}\r\n`;
      this.client.write(data);
    });
  }

  get(key) {
    return new Promise((resolve, reject) => {
      const requestId = (this.requestId++).toString();

      // Store the promise resolve/reject functions
      this.pendingRequests.set(requestId, { resolve, reject });

      // Set a timeout in case the server doesn't respond
      const timeout = setTimeout(() => {
        if (this.pendingRequests.has(requestId)) {
          this.pendingRequests.delete(requestId);
          reject(new Error(`GET operation timed out for key: ${key}`));
        }
      }, 5000);

      // Override the resolve function to clear the timeout
      const originalResolve = resolve;
      resolve = (response) => {
        clearTimeout(timeout);
        originalResolve(response);
      };

      const data = `GET ${key}\n\r\n`;
      this.client.write(data);
    });
  }
}

module.exports = {
  connect: async (localhost = "localhost", port = 6411) => {
    const client = new MdisClient(localhost, port);
    await client.connect();
    return client;
  },
};
