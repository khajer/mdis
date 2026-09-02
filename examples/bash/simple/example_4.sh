#!/usr/bin/env bash
# Example 4: SET a payload larger than 4096 bytes to exercise chunked
# transfer encoding.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source src/mdis_client.sh

HOST="127.0.0.1"
PORT=6411

txt_data="$(printf 'a%.0s' {1..5000})"

resp="$(mdis_set "$HOST" "$PORT" "token" "$txt_data")"
echo "${resp}"
