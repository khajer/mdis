# Minimal TCP client for the mdis protocol. Stdlib only (socket) — no
# external gems, same spirit as the Python client.
require "socket"

class MdisClient
  MAX_BUFFER_SIZE = 4096

  def self.connect(host = "127.0.0.1", port = 6411)
    new(host, port)
  end

  def initialize(host = "127.0.0.1", port = 6411)
    @host = host
    @port = port
  end

  # Store value under key. duration is the optional per-key expiration in
  # seconds; 0 (default) uses the server's EXPIRE_TIMEOUT.
  def set(key, value, duration = 0)
    value = value.to_s
    headers = +""
    headers << "Duration: #{duration}\r\n" if duration != 0

    payload = value
    if value.bytesize > MAX_BUFFER_SIZE
      headers << "transfer-encoding: chunked\r\n"
      payload = chunk_encode(value)
    end

    send_command("set #{key}\r\n#{headers}\r\n#{payload}\r\n\r\n")
  end

  # Retrieve the value stored under key.
  def get(key)
    send_command("get #{key}\r\n")
  end

  private

  def send_command(message)
    socket = TCPSocket.new(@host, @port)
    socket.write(message)
    socket.close_write
    response = socket.read # server closes the connection after replying
    socket.close
    parse_response(response.to_s)
  end

  # Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
  # chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
  # terminated by "0\r\n\r\n".
  def chunk_encode(data)
    out = +""
    offset = 0
    while offset < data.bytesize
      size = [MAX_BUFFER_SIZE, data.bytesize - offset].min
      out << "#{size.to_s(16)}\r\n#{data.byteslice(offset, size)}\r\n"
      offset += size
    end
    out << "0\r\n\r\n"
  end

  # Mirrors the Node.js/Python client parsing of the same wire format:
  #   SET success:       OK\r\ninsert completed\r\n
  #   GET found:          OK\r\n\r\n{data}\r\n\r\n
  #   GET not found:       OK\r\n\r\n
  #   error/expired key:   Err\r\n
  def parse_response(data)
    parts = data.split("\r\n", -1)
    return "NO RESPONSE" if parts.empty?

    case parts[0].downcase
    when "ok"
      if parts.length >= 3 && parts[1] == ""
        parts[2] || "" # GET with a value
      elsif parts.length >= 2 && parts[1] != ""
        parts[1] # SET response message
      else
        "" # GET, key not found
      end
    when "err"
      "Error"
    else
      "NO RESPONSE"
    end
  end
end
