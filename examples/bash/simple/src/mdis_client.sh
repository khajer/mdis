# Minimal TCP client for the mdis protocol using bash's built-in
# /dev/tcp socket redirection. No external dependencies (no netcat/socat
# required) -- same spirit as the Python and Ruby clients.
#
# Meant to be sourced, not executed:
#   source "$(dirname "$0")/src/mdis_client.sh"

MDIS_MAX_BUFFER_SIZE=4096

# Send a raw request to the server and print the raw response.
# A fresh connection is opened per call -- the server closes the
# connection after replying to a single request (same behavior the
# Python/Ruby clients rely on).
#
# usage: mdis_send host port request
mdis_send() {
  local host="$1" port="$2" request="$3"

  exec 3<>"/dev/tcp/${host}/${port}" || return 1
  printf '%s' "$request" >&3
  # bash has no shutdown(SHUT_WR), so just read until the server
  # closes its end of the connection.
  cat <&3
  exec 3>&-
}

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

# Splits a response on literal \r\n delimiters (mirrors Ruby's
# `data.split("\r\n", -1)`); bash's IFS splitting can't take a
# multi-char separator, so walk the string instead.
mdis_split() {
  local rest="$1" sep=$'\r\n'

  while [[ "$rest" == *"$sep"* ]]; do
    printf '%s\n' "${rest%%"$sep"*}"
    rest="${rest#*"$sep"}"
  done
  printf '%s\n' "$rest"
}

# Mirrors the response parsing done by the other language clients for
# the same wire format:
#   SET success:        OK\r\ninsert completed\r\n
#   GET found:           OK\r\n\r\n{data}\r\n\r\n
#   GET not found:        OK\r\n\r\n
#   error/expired key:    Err\r\n
mdis_parse_response() {
  # $(...) command substitution (used by mdis_send) strips trailing
  # \n but leaves a dangling \r behind (e.g. "Err\r\n" -> "Err\r").
  # Trim it here so it doesn't end up glued onto the last split part.
  local data="${1%$'\r'}"
  local -a parts=()
  local line status

  while IFS= read -r line; do
    parts+=("$line")
  done < <(mdis_split "$data")

  status="$(printf '%s' "${parts[0]:-}" | tr '[:upper:]' '[:lower:]')"

  case "$status" in
    ok)
      if (( ${#parts[@]} >= 3 )) && [[ -z "${parts[1]}" ]]; then
        printf '%s' "${parts[2]}"   # GET with a value
      elif (( ${#parts[@]} >= 2 )) && [[ -n "${parts[1]}" ]]; then
        printf '%s' "${parts[1]}"   # SET response message
      else
        printf ''                   # GET, key not found
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

# usage: mdis_set host port key value [duration_sec]
mdis_set() {
  local host="$1" port="$2" key="$3" value="$4" duration="${5:-0}"
  local headers="" payload="$value" size=${#value} request raw

  if (( duration != 0 )); then
    headers+="Duration: ${duration}"$'\r\n'
  fi

  if (( size > MDIS_MAX_BUFFER_SIZE )); then
    headers+="transfer-encoding: chunked"$'\r\n'
    payload="$(mdis_chunk_encode "$value")"
  fi

  request="set ${key}"$'\r\n'"${headers}"$'\r\n'"${payload}"$'\r\n\r\n'
  raw="$(mdis_send "$host" "$port" "$request")"
  mdis_parse_response "$raw"
}

# usage: mdis_get host port key
mdis_get() {
  local host="$1" port="$2" key="$3" request raw

  request="get ${key}"$'\r\n'
  raw="$(mdis_send "$host" "$port" "$request")"
  mdis_parse_response "$raw"
}
