#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"
HTTP_CODE_FILE=$(mktemp)

trap 'rm -f -- "$HTTP_CODE_FILE"' EXIT HUP INT TERM

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

snapshot() {
  directory=$1
  pattern=$2

  if test -d "$directory"; then
    find "$directory" -maxdepth 1 -type f -name "$pattern" \
      -printf '%f\n' | LC_ALL=C sort
  fi
}

if ! curl --silent --show-error --fail --max-time 2 \
  --output /dev/null "$HEALTH_URL"; then
  fail 'server health check failed'
fi

PARTS_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_BEFORE=$(snapshot "$FINAL_DIRECTORY" '*.final')

if curl --silent --show-error \
  --request POST \
  --header 'Content-Length: 0' \
  --output /dev/null \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

HTTP_CODE=$(tr -d '\r\n' <"$HTTP_CODE_FILE")
PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')

test "$CURL_STATUS" -eq 0 \
  || fail "curl transport failed with status $CURL_STATUS"
test "$HTTP_CODE" = '400' \
  || fail "empty upload returned HTTP $HTTP_CODE instead of 400"
test "$PARTS_AFTER" = "$PARTS_BEFORE" \
  || fail 'empty upload changed the .part file set'
test "$READY_AFTER" = "$READY_BEFORE" \
  || fail 'empty upload changed the .ready file set'
test "$FINALS_AFTER" = "$FINALS_BEFORE" \
  || fail 'empty upload changed the .final file set'

printf 'PASS: empty upload was rejected without creating stored state\n'
