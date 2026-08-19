curl -i http://127.0.0.1:8080/health

# Expected: HTTP `200` with `It's healthy!`.

curl -i http://127.0.0.1:8080/definitely-missing

# Expected: HTTP `404` with the custom fallback body.

curl -i http://127.0.0.1:8080/media/upload

# Expected: HTTP `405 Method Not Allowed` because this route accepts `POST`, not `GET`.
