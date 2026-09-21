#!/usr/bin/env bash
# Example 1: SET a value.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source src/mdis_client.sh

HOST="127.0.0.1"
PORT=6411

resp="$(mdis_set "$HOST" "$PORT" "token" "123456")"
echo "response: ${resp}"
