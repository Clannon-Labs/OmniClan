#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

MEDIA_FILE="$REPOSITORY_ROOT/samples/sample1.mp4"
HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"

ORIGINAL_FINAL_MODE=''
PERMISSIONS_CHANGED=0
HTTP_CODE_FILE=''
RESPONSE_FILE=''

restore_test_environment() {
  if test "$PERMISSIONS_CHANGED" -eq 1; then
    chmod "$ORIGINAL_FINAL_MODE" "$FINAL_DIRECTORY"
  fi

  if test -n "$HTTP_CODE_FILE"; then
    rm -f -- "$HTTP_CODE_FILE"
  fi

  if test -n "$RESPONSE_FILE"; then
    rm -f -- "$RESPONSE_FILE"
  fi
}

trap restore_test_environment EXIT HUP INT TERM

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

new_entries() {
  before=$1
  after=$2

  comm -13 \
    <(printf '%s\n' "$before" | sed '/^$/d') \
    <(printf '%s\n' "$after" | sed '/^$/d')
}

test -f "$MEDIA_FILE" || fail "sample file does not exist: $MEDIA_FILE"
test -d "$TEMP_DIRECTORY" || fail "temporary directory does not exist: $TEMP_DIRECTORY"
test -d "$FINAL_DIRECTORY" || fail "final directory does not exist: $FINAL_DIRECTORY"

if ! curl --silent --show-error --fail --max-time 2 \
  --output /dev/null "$HEALTH_URL"; then
  fail 'server health check failed'
fi

PARTS_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_BEFORE=$(snapshot "$FINAL_DIRECTORY" '*.final')

ORIGINAL_FINAL_MODE=$(stat -c '%a' "$FINAL_DIRECTORY")
chmod a-w "$FINAL_DIRECTORY"
PERMISSIONS_CHANGED=1

if test -w "$FINAL_DIRECTORY"; then
  fail 'final directory is still writable; rename failure cannot be guaranteed'
fi

HTTP_CODE_FILE=$(mktemp)
RESPONSE_FILE=$(mktemp)

if curl --silent --show-error \
  --header 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE" \
  --output "$RESPONSE_FILE" \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

chmod "$ORIGINAL_FINAL_MODE" "$FINAL_DIRECTORY"
PERMISSIONS_CHANGED=0

HTTP_CODE=$(tr -d '\r\n' <"$HTTP_CODE_FILE")
PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')
NEW_READY=$(new_entries "$READY_BEFORE" "$READY_AFTER")
NEW_READY_COUNT=$(printf '%s\n' "$NEW_READY" | sed '/^$/d' | wc -l)

test "$CURL_STATUS" -eq 0 \
  || fail "curl transport failed with status $CURL_STATUS"
test "$HTTP_CODE" = '500' \
  || fail "server returned HTTP $HTTP_CODE instead of 500"
test "$PARTS_AFTER" = "$PARTS_BEFORE" \
  || fail 'the .part file set changed'
test "$FINALS_AFTER" = "$FINALS_BEFORE" \
  || fail 'the .final file set changed'
test "$NEW_READY_COUNT" -eq 1 \
  || fail "expected exactly one new .ready file, found $NEW_READY_COUNT"

READY_FILE="$TEMP_DIRECTORY/$NEW_READY"
test -f "$READY_FILE" || fail "new .ready file does not exist: $READY_FILE"
cmp --silent "$MEDIA_FILE" "$READY_FILE" \
  || fail 'retained .ready bytes differ from the uploaded source'

printf 'PASS: final rename failed and retained the complete upload at %s\n' "$READY_FILE"
