#!/usr/bin/env bash
# Example 3: SET with a per-key expiration, then GET after it lapses.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source src/mdis_client.sh

HOST="127.0.0.1"
PORT=6411

resp="$(mdis_set "$HOST" "$PORT" "token" "123456" 2)"
echo "resp: ${resp}"

sleep 3

token="$(mdis_get "$HOST" "$PORT" "token")"
echo "token: ${token}"
