#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

BASE_URL='http://127.0.0.1:8080'
EXPECTED_FALLBACK="$REPOSITORY_ROOT/backend/crates/server/services/fallback.html"
RESPONSE_FILE=$(mktemp)

trap 'rm -f -- "$RESPONSE_FILE"' EXIT HUP INT TERM

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

request_status() {
  curl --silent --show-error \
    --request POST \
    --output "$RESPONSE_FILE" \
    --write-out '%{http_code}' \
    "$1"
}

CLEANUP_STATUS=$(request_status "$BASE_URL/media/cleanup")

if [ "$CLEANUP_STATUS" != '200' ]; then
  printf 'Response body:\n'
  cat "$RESPONSE_FILE"
  printf '\n'

  fail "cleanup route returned HTTP $CLEANUP_STATUS instead of 200"
fi

if [ "$(cat "$RESPONSE_FILE")" != "Cleanup completed" ]; then
  printf 'Unexpected response body:\n'
  cat "$RESPONSE_FILE"
  printf '\n'

  fail 'cleanup route returned an unexpected body'
fi

printf 'PASS: Cleanup route works perfectly!\n'
