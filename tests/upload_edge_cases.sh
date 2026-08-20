#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"
MAX_UPLOAD_BYTES=3145728
OVERSIZE_FILE=''
HTTP_CODE_FILE=''
RAW_RESPONSE_FILE=''

cleanup_test_files() {
  for test_file in "$OVERSIZE_FILE" "$HTTP_CODE_FILE" "$RAW_RESPONSE_FILE"; do
    if test -n "$test_file"; then
      rm -f -- "$test_file"
    fi
  done
}

trap cleanup_test_files EXIT HUP INT TERM

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

record_storage_state() {
  PARTS_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.part')
  READY_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.ready')
  FINALS_BEFORE=$(snapshot "$FINAL_DIRECTORY" '*.final')
}

assert_storage_unchanged() {
  condition=$1
  parts_after=$(snapshot "$TEMP_DIRECTORY" '*.part')
  ready_after=$(snapshot "$TEMP_DIRECTORY" '*.ready')
  finals_after=$(snapshot "$FINAL_DIRECTORY" '*.final')

  test "$parts_after" = "$PARTS_BEFORE" \
    || fail "$condition changed the .part file set"
  test "$ready_after" = "$READY_BEFORE" \
    || fail "$condition changed the .ready file set"
  test "$finals_after" = "$FINALS_BEFORE" \
    || fail "$condition changed the .final file set"
}

read_http_code() {
  tr -d '\r\n' <"$HTTP_CODE_FILE"
}

if ! curl --silent --show-error --fail --max-time 2 \
  --output /dev/null "$HEALTH_URL"; then
  fail 'server health check failed'
fi

OVERSIZE_FILE=$(mktemp)
HTTP_CODE_FILE=$(mktemp)
RAW_RESPONSE_FILE=$(mktemp)
truncate -s "$((MAX_UPLOAD_BYTES + 1))" "$OVERSIZE_FILE"

record_storage_state

if curl --silent --show-error \
  --header 'Content-Type: application/octet-stream' \
  --data-binary @"$OVERSIZE_FILE" \
  --output /dev/null \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

HTTP_CODE=$(read_http_code)
test "$CURL_STATUS" -eq 0 \
  || fail "header-limit request had curl status $CURL_STATUS"
test "$HTTP_CODE" = '413' \
  || fail "header-limit request returned HTTP $HTTP_CODE instead of 413"
assert_storage_unchanged 'header-limit rejection'
printf 'PASS: oversized Content-Length was rejected before storage changed\n'

record_storage_state

if curl --silent --show-error --http1.1 \
  --header 'Content-Length:' \
  --header 'Transfer-Encoding: chunked' \
  --header 'Content-Type: application/octet-stream' \
  --data-binary @"$OVERSIZE_FILE" \
  --output /dev/null \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

HTTP_CODE=$(read_http_code)
test "$CURL_STATUS" -eq 0 \
  || fail "stream-limit request had curl status $CURL_STATUS"
test "$HTTP_CODE" = '413' \
  || fail "stream-limit request returned HTTP $HTTP_CODE instead of 413"
assert_storage_unchanged 'stream-limit rejection'
printf 'PASS: oversized chunked body was rejected and its partial file removed\n'

record_storage_state

if ! printf 'POST /media/upload HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nContent-Length: nope\r\nConnection: close\r\n\r\n' \
  | timeout 3 nc 127.0.0.1 8080 >"$RAW_RESPONSE_FILE"; then
  fail 'malformed Content-Length request did not complete normally'
fi

FIRST_RESPONSE_LINE=$(sed -n '1{s/\r$//;p;}' "$RAW_RESPONSE_FILE")
case "$FIRST_RESPONSE_LINE" in
  'HTTP/1.1 400 '*) ;;
  *) fail "malformed Content-Length produced: $FIRST_RESPONSE_LINE" ;;
esac
assert_storage_unchanged 'malformed Content-Length rejection'
printf 'PASS: malformed Content-Length was rejected by the HTTP boundary\n'

record_storage_state

if ! printf 'POST /media/upload HTTP/1.1\r\nHost: 127.0.0.1:8080\r\nContent-Length: 3\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n0\r\n\r\n' \
  | timeout 3 nc 127.0.0.1 8080 >"$RAW_RESPONSE_FILE"; then
  fail 'conflicting framing request did not complete normally'
fi

FIRST_RESPONSE_LINE=$(sed -n '1{s/\r$//;p;}' "$RAW_RESPONSE_FILE")
case "$FIRST_RESPONSE_LINE" in
  'HTTP/1.1 400 '*) ;;
  '')
    if ! curl --silent --show-error --fail --max-time 2 \
      --output /dev/null "$HEALTH_URL"; then
      fail 'conflicting framing closed the connection and left the server unhealthy'
    fi
    ;;
  *) fail "conflicting framing produced: $FIRST_RESPONSE_LINE" ;;
esac

PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')
NEW_PART=$(new_entries "$PARTS_BEFORE" "$PARTS_AFTER")
NEW_PART_COUNT=$(printf '%s\n' "$NEW_PART" | sed '/^$/d' | wc -l)

test "$READY_AFTER" = "$READY_BEFORE" \
  || fail 'conflicting framing changed the .ready file set'
test "$FINALS_AFTER" = "$FINALS_BEFORE" \
  || fail 'conflicting framing changed the .final file set'
test "$NEW_PART_COUNT" -eq 1 \
  || fail "conflicting framing retained $NEW_PART_COUNT new .part files instead of one"

PART_FILE="$TEMP_DIRECTORY/$NEW_PART"
test -f "$PART_FILE" || fail "retained .part file does not exist: $PART_FILE"
cmp --silent <(printf 'abc') "$PART_FILE" \
  || fail 'retained .part does not contain the decoded three-byte body'

printf 'PASS: conflicting framing followed chunked precedence and retained one cleanup-owned .part\n'
