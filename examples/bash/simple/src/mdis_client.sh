# Minimal TCP client for the mdis protocol using bash's built-in
# /dev/tcp socket redirection. No external dependencies (no netcat/socat
# required) -- same spirit as the Python and Ruby clients.
#
# Meant to be sourced, not executed:
#   source "$(dirname "$0")/src/mdis_client.sh"

source "$(dirname "${BASH_SOURCE[0]}")/mdis_protocol.sh"

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

# usage: mdis_set host port key value [duration_sec]
mdis_set() {
  local host="$1" port="$2" key="$3" value="$4" duration="${5:-0}"
  local raw

  mdis_build_set_request "$key" "$value" "$duration"
  raw="$(mdis_send "$host" "$port" "$REPLY")"
  mdis_parse_response "$raw"
}

# usage: mdis_get host port key
mdis_get() {
  local host="$1" port="$2" key="$3" request raw

  request="get ${key}"$'\r\n'
  raw="$(mdis_send "$host" "$port" "$request")"
  mdis_parse_response "$raw"
}
