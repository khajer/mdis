const MdisClient = require('../src/index.js');
const net = require('net');
const { spawn } = require('child_process');

// Mock server that implements the protocol as defined in shared.rs
function createMockServer() {
  const server = net.createServer((socket) => {
    let dataBuffer = '';

    socket.on('data', (data) => {
      dataBuffer += data.toString();

      // Process the complete message when we have enough data
      if (dataBuffer.includes('\r\n') || dataBuffer.endsWith('\n')) {
        const message = dataBuffer.trim();
        dataBuffer = ''; // Reset buffer

        // Parse the message according to the Rust protocol
        const parts = message.split('\r\n');
        const header = parts[0];
        const headerParts = header.split(' ');

        if (headerParts.length >= 2) {
          const method = headerParts[0].toLowerCase();

          if (method === 'set') {
            const key = headerParts[1];
            const value = parts[1] || '';

            // Handle optional expiration
            let responseText = 'insert completed';
            if (headerParts.length === 3) {
              // Expiration provided
              const exp = headerParts[2];
              responseText = `insert completed with exp=${exp}`;
            }

            socket.write(`OK\r\n${responseText}\r\n`);
          } else if (method === 'get') {
            const key = headerParts[1];

            // Simulate different scenarios based on key
            if (key === 'key1') {
              socket.write(`OK\r\nvalue1\r\n`);
            } else if (key === 'key2') {
              socket.write(`OK\r\nvalue2\r\n`);
            } else if (key === 'expired') {
              // Simulate expired key
              socket.write(`Err\r\n`);
            } else {
              // Key doesn't exist
              socket.write(`OK\r\n\r\n`);
            }
          } else {
            socket.write(`Err\r\n`);
          }
        } else {
          socket.write(`Err\r\n`);
        }
      }
    });

    socket.on('close', () => {
      // Connection closed
    });
  });

  return server;
}

describe('MdisClient', () => {
  let mockServer;
  const port = 16411; // Use a different port to avoid conflicts

  beforeAll((done) => {
    mockServer = createMockServer();
    mockServer.listen(port, '127.0.0.1', done);
  });

  afterAll((done) => {
    mockServer.close(done);
  });

  test('should set a key without expiration', async () => {
    const client = MdisClient.connect('127.0.0.1', port);
    const result = await client.set('key1', 'value1');
    expect(result).toBe('insert completed');
  });

  test('should set a key with expiration', async () => {
    const client = MdisClient.connect('127.0.0.1', port);
    const result = await client.set('key2', 'value2', 300);
    expect(result).toBe('insert completed with exp=300');
  });

  test('should get a value for existing key', async () => {
    const client = MdisClient.connect('127.0.0.1', port);
    const result = await client.get('key1');
    expect(result).toBe('value1');
  });

  test('should return empty string for non-existent key', async () => {
    const client = MdisClient.connect('127.0.0.1', port);
    const result = await client.get('nonexistent');
    expect(result).toBe('');
  });

  test('should handle error for expired key', async () => {
    const client = MdisClient.connect('127.0.0.1', port);
    const result = await client.get('expired');
    expect(result).toBe('Error');
  });
});
