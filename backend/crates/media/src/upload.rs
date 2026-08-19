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
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt
};

// 3MB limit
const MAX_UPLOAD_BYTES: u64 = 3*1024*1024; // 1MB = 1,048,576

pub async fn handle_media_upload(request: Request) -> (StatusCode, String) {
    let ok = StatusCode::OK;
    
    if let Err(err) = handle_media_header(&request) {
        return err;
    }

    let (bytes, filepath) = match handle_media_body(request).await {
        Ok((bytes, filepath)) => (bytes, filepath),
        Err(err) => return err,
    };

    (
        ok,
        format!("{ok}: File with bytes {bytes} was created on {filepath}!\n")
    )
    
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

async fn handle_media_body(request: Request) -> Result<(u64, String), (StatusCode, String)> {
    let payload_too_large = StatusCode::PAYLOAD_TOO_LARGE;
    let bad_request = StatusCode::BAD_REQUEST;
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;
    
    let body = request.into_body();
    let mut stream = body.into_data_stream();

    let mut total_bytes: u64 = 0;

    // relative to where code is ran from
    let mut path = std::path::PathBuf::from("./uploads"); 
    match fs::create_dir_all(&path).await {
        Ok(()) => {},
        Err(_) => {
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Couldn't create the required directories!\n")
            ))
        }
    }
    // until the file is fully processed, its name
    // will have .part, and .final after it's fully verified
    // and written down
    let filename = uuid::Uuid::now_v7();
    path.push(format!("{filename}.part"));

    let mut file = match File::create(&path).await {
        Ok(f) => f,
        _ => {
            let filepath = path.display();
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Couldn't create file '{filepath}'!\n")
            ))
        }
    };
    
    while let Some(stream_chunk) = stream.next().await {
        match stream_chunk {
            Ok(chunk) => {
                match total_bytes.checked_add(chunk.len() as u64) {
                    Some(new_total) if new_total <= MAX_UPLOAD_BYTES => {
                        total_bytes = new_total;
                        
                        match file.write_all(&chunk).await {
                            Ok(_) => {},
                            _ => {
                                let filepath = path.display();
                                return Err((
                                    internal_server_error,
                                    format!("{internal_server_error}: Couldn't write bytes to file '{filepath}'!\n")
                                ))
                            }
                        }
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

    match file.flush().await {
        Ok(()) => {
            // all bytes are fully written,
            // so change the name to .final
            // 
            // media handler doesn't automatically sort out the
            // stored files yet
            let old_path = path.clone();
            path.set_file_name(format!("{filename}.final"));

            match tokio::fs::rename(&old_path, &path).await {
                Ok(_) => Ok((
                    total_bytes,
                    path.display().to_string()
                )),
                Err(_) => {
                    return Err((
                        internal_server_error,
                        format!("{internal_server_error}: Couldn't finalize the name of the file!\n")
                    ))
                }
            }
        },
        Err(e) => {
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Error '{e}' while flushing bytes into file!\n")
            ))
        }
    }
}
