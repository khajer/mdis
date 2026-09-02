#!/usr/bin/env bash
# Example 2: GET a value.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source src/mdis_client.sh

HOST="127.0.0.1"
PORT=6411

token="$(mdis_get "$HOST" "$PORT" "token")"
echo "token: ${token}"
