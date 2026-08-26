use axum::{
    extract::Request,
    http::{StatusCode, header},
};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use std::path::PathBuf;
// Couldn't create the required directories!
// 3MB limit
const MAX_UPLOAD_BYTES: u64 = 3 * 1024 * 1024; // 1MB = 1,048,576

use crate::utils;

pub async fn handle_media_upload(request: Request) -> (StatusCode, String) {
    let ok = StatusCode::OK;

    if let Err(err) = handle_media_header(&request) {
        // println!("{:?}", err);
        return err;
    }

    let (bytes, filepath) = match handle_media_body(request).await {
        Ok((bytes, filepath)) => {
            // println!("Bytes: {}, Filepath: {}", bytes, filepath);
            (bytes, filepath)
        }
        Err(err) => {
            // println!("{:?}", err);
            return err;
        }
    };

    (
        ok,
        format!("{ok}: File with bytes {bytes} was created on {filepath}!\n"),
    )
}

fn handle_media_header(request: &Request) -> Result<(), (StatusCode, String)> {
    let payload_too_large = StatusCode::PAYLOAD_TOO_LARGE;
    let bad_request = StatusCode::BAD_REQUEST;

    let headers = request.headers();

    match headers.get(header::CONTENT_LENGTH) {
        Some(length) => match length.to_str() {
            Ok(len_str) => match len_str.parse::<u64>() {
                Ok(len) if len > MAX_UPLOAD_BYTES => {
                    return Err((
                        payload_too_large,
                        format!("{payload_too_large}: Parsed payload limit exceeded!\n"),
                    ));
                }
                Err(err) => return Err((bad_request, format!("{bad_request}: {err}\n"))),
                Ok(_) => Ok(()),
            },
            Err(err) => return Err((bad_request, format!("{bad_request}: {err}\n"))),
        },
        None => Ok(()),
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
    let final_location = crate::constants::FINAL_LOCATION;
    let temp_location = crate::constants::TEMP_LOCATION;
    let mut final_path = PathBuf::from(format!("{final_location}"));
    let mut temp_path = PathBuf::from(format!("{temp_location}"));

    utils::create_dir(&temp_path).await?;
    // until the file is fully processed, its name
    // will have .part, and .final after it's fully verified
    // and written down
    let filename = uuid::Uuid::now_v7();

    // create file in temp directory first
    let part_file = crate::constants::PARTIAL_FILE;
    temp_path.push(format!("{filename}.{part_file}"));

    let mut file = utils::create_file(&temp_path).await?;

    while let Some(stream_chunk) = stream.next().await {
        match stream_chunk {
            Ok(chunk) => {
                match total_bytes.checked_add(chunk.len() as u64) {
                    Some(new_total) if new_total <= MAX_UPLOAD_BYTES => {
                        total_bytes = new_total;

                        match file.write_all(&chunk).await {
                            Ok(_) => {}
                            _ => {
                                let filepath = temp_path.display();
                                utils::remove(&temp_path).await?;
                                // println!("{internal_server_error}: Couldn't write bytes to file '{filepath}'!\n");
                                return Err((
                                    internal_server_error,
                                    format!(
                                        "{internal_server_error}: Couldn't write bytes to file '{filepath}'!\n"
                                    ),
                                ));
                            }
                        }
                    }
                    _ => {
                        utils::remove(&temp_path).await?;
                        return Err((
                            payload_too_large,
                            format!("{payload_too_large}: Payload limit exceeded!\n"),
                        ));
                    }
                }
            }
            Err(_) => {
                utils::remove(&temp_path).await?;
                return Err((
                    bad_request,
                    format!("{bad_request}: Invalid payload body!\n"),
                ));
            }
        }
    }

    final_check(&mut file, &filename, &temp_path, &mut final_path).await?;

    return Ok((total_bytes, final_path.display().to_string()));
}

async fn final_check(
    file: &mut tokio::fs::File,
    filename: &uuid::Uuid,
    temp_path: &PathBuf,
    // ready_path: &PathBuf, // no need to give ready path, it will make it on its own with filename
    final_path: &mut PathBuf,
) -> Result<(), (StatusCode, String)> {
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;
    let bad_request = StatusCode::BAD_REQUEST;

    match file.flush().await {
        Ok(_) => {
            match file.sync_all().await {
                Ok(_) => {
                    // we dont want to keep empty file at all,
                    // not even in temp directory, we delete if
                    // it's empty
                    if utils::is_empty(&temp_path).await {
                        utils::remove(&temp_path).await?;

                        // println!("Temporary file is found to be empty");
                        return Err((
                            bad_request,
                            format!("{bad_request}: Temporary file is empty!\n"),
                        ));
                    }

                    // if it's not empty and written to disk, change it to be .ready to commit it
                    let mut ready_path = temp_path.clone();
                    let ready_file = crate::constants::READY_FILE;
                    ready_path.set_file_name(format!("{filename}.{ready_file}"));

                    utils::rename(temp_path, &ready_path).await?;

                    // all bytes are fully written,
                    // so change the name to .final in memory,
                    // which will be changed to .final in disk upon move

                    // media handler doesn't automatically sort out the
                    // files, it's the job of the db.

                    // Media handler just tags file as .final as confirmed
                    // and makes it a lil easier by making them indexable directly
                    // by their name, but DB's created_at is still authoritative

                    utils::create_dir(&final_path).await?;
                    let final_file = crate::constants::FINAL_FILE;
                    
                    final_path.push(format!("{filename}.{final_file}"));

                    utils::rename(&ready_path, &final_path).await?;

                    return Ok(());
                }
                Err(e) => {
                    let temp = temp_path.display().to_string();
                    // println!("{internal_server_error}: Err '{e}' occured while writing file '{temp}' to disk!\n");
                    return Err((
                        internal_server_error,
                        format!(
                            "{internal_server_error}: Err '{e}' occured while writing file '{temp}' to disk!\n"
                        ),
                    ));
                }
            }
        }
        Err(e) => {
            // println!("{internal_server_error}: Err '{e}' occured while flushing bytes from buffer!\n");
            return Err((
                internal_server_error,
                format!(
                    "{internal_server_error}: Err '{e}' occured while flushing bytes from buffer!\n"
                ),
            ));
        }
    }
}

