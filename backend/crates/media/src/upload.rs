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
    io::AsyncWriteExt
};

// 3MB limit
const MAX_UPLOAD_BYTES: u64 = 3*1024*1024; // 1MB = 1,048,576

pub async fn handle_media_upload(request: Request) -> (StatusCode, String) {
    let ok = StatusCode::OK;
    
    if let Err(err) = handle_media_header(&request) {
        // println!("{:?}", err);
        return err;
    }

    let (bytes, filepath) = match handle_media_body(request).await {
        Ok((bytes, filepath)) =>{
            // println!("Bytes: {}, Filepath: {}", bytes, filepath);
             (bytes, filepath)
        },
        Err(err) => {
            // println!("{:?}", err);
            return err
        },
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
    let mut path = std::path::PathBuf::from("./uploads/media"); 
    let mut temp_path = std::path::PathBuf::from("./uploads/temp");
    
    match tokio::fs::create_dir_all(&temp_path).await {
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

    // create file in temp directory first
    temp_path.push(format!("{filename}.part"));

    let mut file = match tokio::fs::File::create(&temp_path).await {
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
                                match tokio::fs::remove_file(&temp_path).await {
                                    Ok(_) => {
                                        let filepath = path.display();
                                        return Err((
                                            internal_server_error,
                                            format!(
                                                "{internal_server_error}: Couldn't write bytes to file '{filepath}'!\n
                                                But removal of file from temp directory is successful!\n"
                                            )
                                        ))
                                    },
                                    Err(_) => {
                                        let filepath = path.display();
                                        return Err((
                                            internal_server_error,
                                            format!(
                                                "{internal_server_error}: Couldn't write bytes to file '{filepath}'!\n
                                                Also couldn't remove file from temp directory!\n"
                                            )
                                        ))
                                    }
                                }
                            }
                        }
                    },
                    _ => {
                        match tokio::fs::remove_file(&temp_path).await {
                            Ok(_) => {
                                return Err((
                                    payload_too_large,
                                    format!(
                                        "{payload_too_large}: Payload limit exceeded!\n
                                        But removal of file from temp directory was successful!\n"
                                    )
                                ))
                            },
                            _ => {
                                return Err((
                                    payload_too_large,
                                    format!(
                                        "{payload_too_large}: Payload limit exceeded!\n
                                        Also couldn't remove file from temp directory!\n"
                                    )
                                ))
                            }
                        }
                    }
                }
            },
            Err(_) => {
                match tokio::fs::remove_file(&temp_path).await {
                    Ok(_) => {
                        return Err((
                            bad_request,
                            format!(
                                "{bad_request}: Invalid payload body!\n
                                But removal of file from temp directory was successful\n"
                            )
                        ))
                    },
                    _ => {
                        return Err((
                            bad_request,
                            format!(
                                "{bad_request}: Invalid payload body!\n
                                Also couldn't remove file from temp directory\n"
                            )
                        ))
                    }
                }
            }
        }
    }

    match file.flush().await {
        Ok(()) => {
            // all bytes are fully written,
            // so change the name to .final in memory,
            // which will be changed to .final in disk upon move
            // 
            // media handler doesn't automatically sort out the
            // files, it's the job of the db.
            // 
            // Media handler just tags file as .final as confirmed
            // and makes it a lil easier by making them indexable directly
            // by their name, but DB's created_at is still authoritative
            path.set_file_name(format!("{filename}.final"));

            // Move the file to /media directory
            match tokio::fs::rename(&temp_path, &path).await {
                Ok(_) => {
                    // confirmed the file is moved into /media
                    // directory, so we dont need to try to
                    // delete it from /temp directory
                    Ok((
                        total_bytes,
                        path.display().to_string()
                    ))
                },
                Err(_) => {
                    let temp = temp_path.display().to_string();
                    return Err((
                        internal_server_error,
                        format!("{internal_server_error}: Couldn't move file '{temp}' into permanent directory!\n")
                    ))
                }
            }
        },
        Err(e) => {
            let temp = temp_path.display().to_string();
            match tokio::fs::remove_file(&temp_path).await{
                Ok(_) => {
                    return Err((
                        internal_server_error,
                        format!(
                            "{internal_server_error}: Error '{e}' while flushing bytes into file!\n
                            But removal of file {temp} successful!\n"
                        )
                    ))
                },
                Err(_) => {
                    return Err((
                        internal_server_error,
                        format!(
                            "{internal_server_error}: Error '{e}' while flushing bytes into file!\n
                            Error occured also while removing the file {temp}!\n"
                        )
                    ))
                }
            }
        }
    }
}
