#!/usr/bin/env bash
set -euo pipefail

TEST_SCRIPT_DIRECTORY=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(CDPATH= cd -- "$TEST_SCRIPT_DIRECTORY/.." && pwd)

MEDIA_FILE="$REPOSITORY_ROOT/samples/sample1.mp4"
HEALTH_URL='http://127.0.0.1:8080/health'
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
TEMP_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/temp"
FINAL_DIRECTORY="$REPOSITORY_ROOT/backend/uploads/media"

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

test -f "$MEDIA_FILE" || fail "sample file does not exist: $MEDIA_FILE"

if ! curl --silent --show-error --fail --max-time 2 \
  --output /dev/null "$HEALTH_URL"; then
  fail 'server health check failed'
fi

PARTS_BEFORE=$(snapshot "$TEMP_DIRECTORY" '*.part')
FINALS_BEFORE=$(snapshot "$FINAL_DIRECTORY" '*.final')

CURL_RESULT_FILE=$(mktemp)
trap 'rm -f -- "$CURL_RESULT_FILE"' EXIT HUP INT TERM

curl --silent --show-error \
  --max-time 1 \
  --limit-rate 100K \
  --header 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE" \
  --output /dev/null \
  --write-out '%{size_upload}\n' \
  "$UPLOAD_URL" >"$CURL_RESULT_FILE" &
CURL_PID=$!

SAW_NEW_PART=0
SAW_NEW_FINAL=0

while kill -0 "$CURL_PID" 2>/dev/null; do
  CURRENT_PARTS=$(snapshot "$TEMP_DIRECTORY" '*.part')
  CURRENT_FINALS=$(snapshot "$FINAL_DIRECTORY" '*.final')

  if test "$CURRENT_PARTS" != "$PARTS_BEFORE"; then
    SAW_NEW_PART=1
  fi

  if test "$CURRENT_FINALS" != "$FINALS_BEFORE"; then
    SAW_NEW_FINAL=1
  fi

  sleep 0.02
done

if wait "$CURL_PID"; then
  CURL_STATUS=0
else
  CURL_STATUS=$?
fi

CURRENT_PARTS=$(snapshot "$TEMP_DIRECTORY" '*.part')
CURRENT_FINALS=$(snapshot "$FINAL_DIRECTORY" '*.final')

if test "$CURRENT_PARTS" != "$PARTS_BEFORE"; then
  SAW_NEW_PART=1
fi

if test "$CURRENT_FINALS" != "$FINALS_BEFORE"; then
  SAW_NEW_FINAL=1
fi

UPLOAD_BYTES=$(tr -d '\r\n' <"$CURL_RESULT_FILE")
SOURCE_BYTES=$(stat -c '%s' "$MEDIA_FILE")

test "$CURL_STATUS" -eq 28 \
  || fail "curl exited with $CURL_STATUS instead of timeout status 28"

case "$UPLOAD_BYTES" in
  ''|*[!0-9]*) fail "curl reported an invalid uploaded-byte count: $UPLOAD_BYTES" ;;
esac

test "$UPLOAD_BYTES" -gt 0 \
  || fail 'curl did not upload any bytes before disconnecting'
test "$UPLOAD_BYTES" -lt "$SOURCE_BYTES" \
  || fail 'curl sent the complete source file instead of interrupting it'
test "$SAW_NEW_PART" -eq 1 \
  || fail 'no new .part file was observed during the upload'
test "$SAW_NEW_FINAL" -eq 0 \
  || fail 'a new .final file appeared during an interrupted upload'

ATTEMPT=0
while test "$ATTEMPT" -lt 50; do
  PARTS_AFTER=$(snapshot "$TEMP_DIRECTORY" '*.part')
  FINALS_AFTER=$(snapshot "$FINAL_DIRECTORY" '*.final')

  if test "$PARTS_AFTER" = "$PARTS_BEFORE" \
    && test "$FINALS_AFTER" = "$FINALS_BEFORE"; then
    printf 'PASS: interrupted upload created a temporary file, then removed it without committing a final file\n'
    exit 0
  fi

  ATTEMPT=$((ATTEMPT + 1))
  sleep 0.1
done

printf 'Partial files before:\n%s\n' "$PARTS_BEFORE" >&2
printf 'Partial files after:\n%s\n' "$PARTS_AFTER" >&2
printf 'Final files before:\n%s\n' "$FINALS_BEFORE" >&2
printf 'Final files after:\n%s\n' "$FINALS_AFTER" >&2
fail 'stored-file sets did not return to their original state'
