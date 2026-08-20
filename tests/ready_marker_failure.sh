#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

MEDIA_FILE="$REPOSITORY_ROOT/samples/sample1.mp4"
HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"

ORIGINAL_TEMP_MODE=''
PERMISSIONS_CHANGED=0
HTTP_CODE_FILE=''
RESPONSE_FILE=''

restore_test_environment() {
  if test "$PERMISSIONS_CHANGED" -eq 1; then
    chmod "$ORIGINAL_TEMP_MODE" "$TEMP_DIRECTORY"
  fi

  for test_file in "$HTTP_CODE_FILE" "$RESPONSE_FILE"; do
    if test -n "$test_file"; then
      rm -f -- "$test_file"
    fi
  done
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

ORIGINAL_TEMP_MODE=$(stat -c '%a' "$TEMP_DIRECTORY")
HTTP_CODE_FILE=$(mktemp)
RESPONSE_FILE=$(mktemp)

curl --silent --show-error \
  --limit-rate 100K \
  --header 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE" \
  --output "$RESPONSE_FILE" \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE" &
CURL_PID=$!

NEW_PART_OBSERVED=0

while kill -0 "$CURL_PID" 2>/dev/null; do
  CURRENT_PARTS=$(snapshot "$TEMP_DIRECTORY" '*.part')

  if test "$CURRENT_PARTS" != "$PARTS_BEFORE"; then
    NEW_PART_OBSERVED=1
    chmod a-w "$TEMP_DIRECTORY"
    PERMISSIONS_CHANGED=1
    break
  fi

  sleep 0.02
done

if wait "$CURL_PID"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

if test "$PERMISSIONS_CHANGED" -eq 1; then
  chmod "$ORIGINAL_TEMP_MODE" "$TEMP_DIRECTORY"
  PERMISSIONS_CHANGED=0
fi

test "$NEW_PART_OBSERVED" -eq 1 \
  || fail 'no new .part file was observed before curl completed'

HTTP_CODE=$(tr -d '\r\n' <"$HTTP_CODE_FILE")
PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')
NEW_PART=$(new_entries "$PARTS_BEFORE" "$PARTS_AFTER")
NEW_PART_COUNT=$(printf '%s\n' "$NEW_PART" | sed '/^$/d' | wc -l)

test "$CURL_STATUS" -eq 0 \
  || fail "curl transport failed with status $CURL_STATUS"
test "$HTTP_CODE" = '500' \
  || fail "server returned HTTP $HTTP_CODE instead of 500"
test "$READY_AFTER" = "$READY_BEFORE" \
  || fail 'failed ready transition changed the .ready file set'
test "$FINALS_AFTER" = "$FINALS_BEFORE" \
  || fail 'failed ready transition changed the .final file set'
test "$NEW_PART_COUNT" -eq 1 \
  || fail "expected exactly one retained .part file, found $NEW_PART_COUNT"

PART_FILE="$TEMP_DIRECTORY/$NEW_PART"
test -f "$PART_FILE" || fail "retained .part file does not exist: $PART_FILE"
cmp --silent "$MEDIA_FILE" "$PART_FILE" \
  || fail 'retained .part bytes differ from the completely uploaded source'

printf 'PASS: .part to .ready rename failed and retained the completed .part at %s\n' "$PART_FILE"
