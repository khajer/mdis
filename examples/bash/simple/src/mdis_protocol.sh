# Wire-format helpers for the mdis protocol (no I/O). Sourced by
# mdis_client.sh.

MDIS_MAX_BUFFER_SIZE=4096

# Splits data into MDIS_MAX_BUFFER_SIZE chunks using the server's
# hex-size chunked transfer encoding: "{hex_size}\r\n{chunk}\r\n"
# repeated, terminated by "0\r\n\r\n".
mdis_chunk_encode() {
  local data="$1"
  local size=${#data} offset=0 out="" n hex chunk

  while (( offset < size )); do
    n=$(( size - offset < MDIS_MAX_BUFFER_SIZE ? size - offset : MDIS_MAX_BUFFER_SIZE ))
    chunk="${data:offset:n}"
    printf -v hex '%x' "$n"
    out+="${hex}"$'\r\n'"${chunk}"$'\r\n'
    offset=$(( offset + n ))
  done
  out+="0"$'\r\n'$'\r\n'
  printf '%s' "$out"
}

# Decodes a chunked response body back into the original value. Mirrors
# mdis_chunk_encode's framing: "{hex_size}\r\n{chunk}\r\n" repeated,
# terminated by "0\r\n\r\n". `body` is everything after the
# "OK\r\ntransfer-encoding: chunked\r\n\r\n" prefix.
mdis_chunk_decode() {
  local body="$1" offset=0 out="" rest size_str size chunk

  while :; do
    rest="${body:offset}"
    [[ "$rest" == *$'\r\n'* ]] || break
    size_str="${rest%%$'\r\n'*}"
    size=$((16#$size_str))
    (( size == 0 )) && break
    offset=$(( offset + ${#size_str} + 2 ))
    chunk="${body:offset:size}"
    out+="$chunk"
    offset=$(( offset + size + 2 ))
  done
  printf '%s' "$out"
}

# Mirrors the response parsing done by the other language clients for
# the same wire format:
#   SET success:        OK\r\ninsert completed\r\n
#   GET found:           OK\r\n\r\n{data}\r\n\r\n
#   GET not found:        OK\r\n\r\n
#   GET found (chunked): OK\r\ntransfer-encoding: chunked\r\n\r\n{chunks}
#   error/expired key:    Err\r\n
mdis_parse_response() {
  # $(...) command substitution (used by mdis_send) strips trailing
  # \n but leaves a dangling \r behind (e.g. "Err\r\n" -> "Err\r").
  # Trim it here so it doesn't end up glued onto the last split part.
  local data="${1%$'\r'}"
  local chunked_prefix=$'OK\r\ntransfer-encoding: chunked\r\n\r\n'

  # A large value comes back chunk-framed rather than as a plain
  # "OK\r\n\r\n{data}\r\n\r\n" body; the chunk data itself may contain
  # \r\n, so it must be decoded from the raw string, not the
  # line-split array used below for the other response shapes.
  if [[ "$data" == "$chunked_prefix"* ]]; then
    mdis_chunk_decode "${data#"$chunked_prefix"}"
    return
  fi

  # Split on literal \r\n (bash's IFS can't take a multi-char separator).
  local -a parts=()
  local rest="$data" sep=$'\r\n' status

  while [[ "$rest" == *"$sep"* ]]; do
    parts+=("${rest%%"$sep"*}")
    rest="${rest#*"$sep"}"
  done
  parts+=("$rest")

  status="$(printf '%s' "${parts[0]:-}" | tr '[:upper:]' '[:lower:]')"

  case "$status" in
    ok)
      if (( ${#parts[@]} >= 3 )) && [[ -z "${parts[1]}" ]]; then
        printf '%s' "${parts[2]}"   # GET with a value
      else
        printf '%s' "${parts[1]:-}" # SET message, or "" for GET not found
      fi
      ;;
    err)
      printf 'Error'
      ;;
    *)
      printf 'NO RESPONSE'
      ;;
  esac
}

# Builds a SET request into REPLY (not stdout: $(...) would strip the
# request's trailing newline). duration 0 uses the server's EXPIRE_TIMEOUT.
#
# usage: mdis_build_set_request key value duration_sec
mdis_build_set_request() {
  local key="$1" value="$2" duration="$3"
  local headers="" payload="$value"

  if (( duration != 0 )); then
    headers+="Duration: ${duration}"$'\r\n'
  fi

  if (( ${#value} > MDIS_MAX_BUFFER_SIZE )); then
    headers+="transfer-encoding: chunked"$'\r\n'
    payload="$(mdis_chunk_encode "$value")"
  fi

  REPLY="set ${key}"$'\r\n'"${headers}"$'\r\n'"${payload}"$'\r\n\r\n'
}
