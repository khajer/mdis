# Wire-format helpers for the mdis protocol (no I/O).
module MdisProtocol
  MAX_BUFFER_SIZE = 4096
  CHUNKED_PREFIX = "OK\r\ntransfer-encoding: chunked\r\n\r\n"

  module_function

  # Splits data into MAX_BUFFER_SIZE chunks using the server's hex-size
  # chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n" repeated,
  # terminated by "0\r\n\r\n".
  def chunk_encode(data)
    out = +""
    (0...data.bytesize).step(MAX_BUFFER_SIZE) do |offset|
      chunk = data.byteslice(offset, MAX_BUFFER_SIZE)
      out << "#{chunk.bytesize.to_s(16)}\r\n#{chunk}\r\n"
    end
    out << "0\r\n\r\n"
  end

  # Decodes a chunked response body back into the original value. `body`
  # is everything after CHUNKED_PREFIX.
  def chunk_decode(body)
    out = +""
    offset = 0
    while (nl = body.index("\r\n", offset))
      size = body[offset...nl].to_i(16)
      break if size.zero?

      data_start = nl + 2
      out << body.byteslice(data_start, size)
      offset = data_start + size + 2 # skip the chunk's trailing \r\n
    end
    out
  end

  # Builds a SET request. duration 0 uses the server's EXPIRE_TIMEOUT.
  def build_set_message(key, value, duration = 0)
    value = value.to_s
    headers = +""
    headers << "Duration: #{duration}\r\n" if duration != 0

    payload = value
    if value.bytesize > MAX_BUFFER_SIZE
      headers << "transfer-encoding: chunked\r\n"
      payload = chunk_encode(value)
    end

    "set #{key}\r\n#{headers}\r\n#{payload}\r\n\r\n"
  end

  # Parses a raw server response:
  #   SET success:         OK\r\ninsert completed\r\n
  #   GET found:           OK\r\n\r\n{data}\r\n\r\n
  #   GET not found:       OK\r\n\r\n
  #   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
  #   error/expired key:   Err\r\n
  def parse_response(data)
    # Chunk data may itself contain \r\n, so a chunked response must be
    # decoded from the raw string, not the \r\n-split array below.
    return chunk_decode(data[CHUNKED_PREFIX.bytesize..]) if data.start_with?(CHUNKED_PREFIX)

    parts = data.split("\r\n", -1)

    case parts[0].to_s.downcase
    when "ok"
      if parts.length >= 3 && parts[1] == ""
        parts[2] # GET with a value
      else
        parts[1].to_s # SET message, or "" for GET not found
      end
    when "err"
      "Error"
    else
      "NO RESPONSE"
    end
  end
end
