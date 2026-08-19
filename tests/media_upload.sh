MEDIA_FILE='../samples/sample2.mp4'

curl -i \
  -H 'Content-Type: video/mp4' \
  --data-binary @"$MEDIA_FILE" \
  http://127.0.0.1:8080/media/upload
