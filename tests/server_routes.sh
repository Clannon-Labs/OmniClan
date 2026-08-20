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
    --output "$RESPONSE_FILE" \
    --write-out '%{http_code}' \
    "$1"
}

HEALTH_STATUS=$(request_status "$BASE_URL/health")
test "$HEALTH_STATUS" = '200' \
  || fail "health route returned HTTP $HEALTH_STATUS instead of 200"
test "$(cat "$RESPONSE_FILE")" = "It's healthy!" \
  || fail 'health route returned an unexpected body'

MISSING_STATUS=$(request_status "$BASE_URL/definitely-missing")
test "$MISSING_STATUS" = '404' \
  || fail "unknown route returned HTTP $MISSING_STATUS instead of 404"
cmp --silent "$EXPECTED_FALLBACK" "$RESPONSE_FILE" \
  || fail 'unknown route did not return the configured fallback document'

METHOD_STATUS=$(request_status "$BASE_URL/media/upload")
test "$METHOD_STATUS" = '405' \
  || fail "GET upload route returned HTTP $METHOD_STATUS instead of 405"

printf 'PASS: health, fallback, and method-routing contracts hold\n'
