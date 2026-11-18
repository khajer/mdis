// Wire-format helpers for the mdis protocol (no I/O).

const MAX_BUFFER_SIZE = 4096;
const CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n";

// Splits data into MAX_BUFFER_SIZE chunks: "{hex_size}\r\n{chunk}\r\n"
// repeated (sizes are hex per the server's protocol), terminated by
// "0\r\n\r\n".
function chunkEncode(data) {
  let out = "";
  for (let offset = 0; offset < data.length; offset += MAX_BUFFER_SIZE) {
    const chunk = data.substring(offset, offset + MAX_BUFFER_SIZE);
    out += `${chunk.length.toString(16)}\r\n${chunk}\r\n`;
  }
  return out + "0\r\n\r\n";
}

// Decodes a chunked response body back into the original value. `body`
// is everything after CHUNKED_PREFIX.
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

function buildSetMessage(key, dataInput, expireDuration) {
  const dataStr = dataInput.toString();
  let headers = "";
  let value = dataInput;

  if (dataStr.length > MAX_BUFFER_SIZE) {
    headers += "transfer-encoding: chunked\r\n";
    value = chunkEncode(dataStr);
  }
  if (expireDuration !== undefined) {
    headers += `duration: ${expireDuration}\r\n`;
  }

  return `set ${key}\r\n${headers}\r\n${value}\r\n\r\n`;
}

// Parses a raw server response:
//   SET success:         OK\r\ninsert completed\r\n
//   GET found:           OK\r\n\r\n{data}\r\n\r\n
//   GET not found:       OK\r\n\r\n
//   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
//   error/expired key:   Err\r\n
function parseResponse(data) {
  const response = data.toString();

  // Chunk data may itself contain \r\n, so a chunked response must be
  // decoded from the raw string, not the \r\n-split array below.
  if (response.startsWith(CHUNKED_PREFIX)) {
    return chunkDecode(response.slice(CHUNKED_PREFIX.length));
  }

  const resp = response.split("\r\n");
  const status = resp[0].toLowerCase();

  if (status === "ok") {
    if (resp.length >= 3 && resp[1] === "") return resp[2]; // GET with a value
    if (resp.length >= 2) return resp[1]; // SET message, or "" for GET not found
    return "";
  }
  if (status === "err") return "Error";
  return "NO RESPONSE";
}

module.exports = { buildSetMessage, parseResponse, chunkEncode, chunkDecode };
