# Minimal TCP client for the mdis protocol. Stdlib only (socket) — no
# external gems, same spirit as the Python client.
require "socket"
require_relative "protocol"

class MdisClient
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
    send_command(MdisProtocol.build_set_message(key, value, duration))
  end

  # Retrieve the value stored under key.
  def get(key)
    send_command("get #{key}\r\n")
  end

  private

  def send_command(message)
    TCPSocket.open(@host, @port) do |socket|
      socket.write(message)
      socket.close_write
      # server closes the connection after replying
      MdisProtocol.parse_response(socket.read.to_s)
    end
  end
end
