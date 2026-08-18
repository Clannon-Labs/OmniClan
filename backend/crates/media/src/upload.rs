use axum::{
    extract::{
      Request,  
    },
    http::{
        StatusCode,
        header,
    }  
};
use futures_util::StreamExt;

const MAX_UPLOAD_BYTES: u64 = 1024;

pub async fn handle_media_upload(request: Request) -> (StatusCode, String) {
    let ok = StatusCode::OK;
    match handle_media_header(&request) {
        Ok(_) => {},
        Err(err) => return err,
    }

    match handle_media_body(request).await {
        Ok(bytes) => {
            (
                ok,
                format!("{ok}: Received {bytes} bytes!\n")
            )
        },
        Err(err) => return err
    }
}

fn handle_media_header(request: &Request) -> Result<(), (StatusCode, String)> {
    let payload_too_large = StatusCode::PAYLOAD_TOO_LARGE;
    let bad_request = StatusCode::BAD_REQUEST;
    
    let headers = request.headers();

    match headers.get(header::CONTENT_LENGTH){
        Some(length) => {
            match length.to_str() {
                Ok(len_str) => {
                    match len_str.parse::<u64>() {
                        Ok(len) if len > MAX_UPLOAD_BYTES => {
                            return Err((
                                payload_too_large,
                                format!("{payload_too_large}: Parsed payload limit exceeded!\n")
                            ))
                        },
                        Err(err) => {
                            return Err((
                                bad_request,
                                format!("{bad_request}: {err}\n")
                            ))
                        },
                        Ok(_) => Ok(()),
                    }
                },
                Err(err) => {
                    return Err((
                        bad_request,
                        format!("{bad_request}: {err}\n")
                    ))
                },
            }
        },
        None => {
            Ok(())
        }
    }
}

async fn handle_media_body(request: Request) -> Result<u64, (StatusCode, String)> {
    let payload_too_large = StatusCode::PAYLOAD_TOO_LARGE;
    let bad_request = StatusCode::BAD_REQUEST;
    
    let body = request.into_body();
    let mut stream = body.into_data_stream();

    let mut total_bytes: u64 = 0;

    while let Some(stream_chunk) = stream.next().await {
        match stream_chunk {
            Ok(chunk) => {
                match total_bytes.checked_add(chunk.len() as u64) {
                    Some(new_total) if new_total <= MAX_UPLOAD_BYTES => {
                        total_bytes = new_total
                    },
                    _ => {
                        return Err((
                            payload_too_large,
                            format!("{payload_too_large}: Payload limit exceeded!\n")
                        ))
                    }
                }
            },
            Err(_) => {
                return Err((
                    bad_request,
                    format!("{bad_request}: Invalid payload body!\n")
                ))
            }
        }
    }
    
    Ok(total_bytes)
}
