# Run it from repo root

MEDIA_FILE="samples/sample1.mp4"
UPLOAD_URL='http://127.0.0.1:8080/media/upload'
UPLOAD_DIRECTORY="backend/uploads"

PARTS_BEFORE=$(find "$UPLOAD_DIRECTORY" -maxdepth 1 -type f -name '*.part' \
  -printf '%f\n' | sort)
FINALS_BEFORE=$(find "$UPLOAD_DIRECTORY" -maxdepth 1 -type f -name '*.final' \
  -printf '%f\n' | sort)

curl --max-time 1 \
  --limit-rate 100K \
  -H 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE" \
  "$UPLOAD_URL"

sleep 1

PARTS_AFTER=$(find "$UPLOAD_DIRECTORY" -maxdepth 1 -type f -name '*.part' \
  -printf '%f\n' | sort)
FINALS_AFTER=$(find "$UPLOAD_DIRECTORY" -maxdepth 1 -type f -name '*.final' \
  -printf '%f\n' | sort)

if test "$PARTS_BEFORE" = "$PARTS_AFTER" \
  && test "$FINALS_BEFORE" = "$FINALS_AFTER"; then
  echo 'PASS: interrupted upload left no new partial or final file'
else
  echo 'FAIL: interrupted upload changed the stored-file sets'
  echo 'Partial files before:'
  printf '%s\n' "$PARTS_BEFORE"
  echo 'Partial files after:'
  printf '%s\n' "$PARTS_AFTER"
  echo 'Final files before:'
  printf '%s\n' "$FINALS_BEFORE"
  echo 'Final files after:'
  printf '%s\n' "$FINALS_AFTER"
fi
