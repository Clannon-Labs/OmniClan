#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

MEDIA_FILE_ONE="$REPOSITORY_ROOT/samples/sample1.mp4"
MEDIA_FILE_TWO="$REPOSITORY_ROOT/samples/sample2.mp4"
HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"
HTTP_CODE_FILE_ONE=''
HTTP_CODE_FILE_TWO=''

cleanup_test_files() {
  for test_file in "$HTTP_CODE_FILE_ONE" "$HTTP_CODE_FILE_TWO"; do
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

test -f "$MEDIA_FILE_ONE" || fail "sample file does not exist: $MEDIA_FILE_ONE"
test -f "$MEDIA_FILE_TWO" || fail "sample file does not exist: $MEDIA_FILE_TWO"

if ! curl --silent --show-error --fail --max-time 2 \
  --output /dev/null "$HEALTH_URL"; then
  fail 'server health check failed'
fi

PARTS_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_BEFORE=$(snapshot "$FINAL_DIRECTORY" '*.final')

HTTP_CODE_FILE_ONE=$(mktemp)
HTTP_CODE_FILE_TWO=$(mktemp)

curl --silent --show-error \
  --header 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE_ONE" \
  --output /dev/null \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE_ONE" &
PID_ONE=$!

curl --silent --show-error \
  --header 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE_TWO" \
  --output /dev/null \
  --write-out '%{http_code}\n' \
  "$UPLOAD_URL" >"$HTTP_CODE_FILE_TWO" &
PID_TWO=$!

if wait "$PID_ONE"; then
  CURL_STATUS_ONE=0
else
  CURL_STATUS_ONE=$?
fi

if wait "$PID_TWO"; then
  CURL_STATUS_TWO=0
else
  CURL_STATUS_TWO=$?
fi

HTTP_CODE_ONE=$(tr -d '\r\n' <"$HTTP_CODE_FILE_ONE")
HTTP_CODE_TWO=$(tr -d '\r\n' <"$HTTP_CODE_FILE_TWO")
PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
READY_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.ready')
FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')
NEW_FINALS=$(new_entries "$FINALS_BEFORE" "$FINALS_AFTER")
NEW_FINAL_COUNT=$(printf '%s\n' "$NEW_FINALS" | sed '/^$/d' | wc -l)

test "$CURL_STATUS_ONE" -eq 0 \
  || fail "first curl transport failed with status $CURL_STATUS_ONE"
test "$CURL_STATUS_TWO" -eq 0 \
  || fail "second curl transport failed with status $CURL_STATUS_TWO"
test "$HTTP_CODE_ONE" = '200' \
  || fail "first upload returned HTTP $HTTP_CODE_ONE instead of 200"
test "$HTTP_CODE_TWO" = '200' \
  || fail "second upload returned HTTP $HTTP_CODE_TWO instead of 200"
test "$PARTS_AFTER" = "$PARTS_BEFORE" \
  || fail 'concurrent uploads changed the .part file set'
test "$READY_AFTER" = "$READY_BEFORE" \
  || fail 'concurrent uploads changed the .ready file set'
test "$NEW_FINAL_COUNT" -eq 2 \
  || fail "expected exactly two new .final files, found $NEW_FINAL_COUNT"

MATCHES_ONE=0
MATCHES_TWO=0

while IFS= read -r final_name; do
  if test -z "$final_name"; then
    continue
  fi

  final_file="$FINAL_DIRECTORY/$final_name"

  if cmp --silent "$MEDIA_FILE_ONE" "$final_file"; then
    MATCHES_ONE=$((MATCHES_ONE + 1))
  fi

  if cmp --silent "$MEDIA_FILE_TWO" "$final_file"; then
    MATCHES_TWO=$((MATCHES_TWO + 1))
  fi
done <<<"$NEW_FINALS"

test "$MATCHES_ONE" -eq 1 \
  || fail "sample1 matched $MATCHES_ONE committed files instead of one"
test "$MATCHES_TWO" -eq 1 \
  || fail "sample2 matched $MATCHES_TWO committed files instead of one"

printf 'PASS: two concurrent uploads produced distinct, byte-identical final files\n'
