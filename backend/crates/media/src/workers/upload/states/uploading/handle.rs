
use std::path::PathBuf;
use tokio::{
    io::AsyncWriteExt,
};
use axum::{
    http::{
        HeaderMap,
        header,
    },
    body::Body,
};
use futures_util::StreamExt;

use super::super::super::utils;

pub(super) async fn handle_uploading(
    headers: &HeaderMap,
    body: Body,
    max_bytes: u64,
    input_path: PathBuf,
) -> Result<u64, std::io::Error> {
    
    handle_media_header(
        headers,
        max_bytes
    )?;
    
    let uploaded_bytes = handle_media_body(
        body,
        max_bytes,
        input_path,
    ).await?;

    Ok(uploaded_bytes)
    
}

fn handle_media_header(
    headers: &HeaderMap,
    max_bytes: u64,
) -> Result<(), std::io::Error> {
    
    match headers.get(header::CONTENT_LENGTH) {
        Some(length) => match length.to_str() {
            Ok(len_str) => match len_str.parse::<u64>() {
                Ok(len) if len > max_bytes => {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Size limit exceeded the threshold!"
                        ));
                },
                Err(_) => {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Couldn't compare the size of the file!"
                        )
                    );
                },
                Ok(_) => Ok(())
            },
            Err(_) => {
                return Err(
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Couldn't parse the header!"
                    )
                );
            }
        },
        None => Ok(())
    }
}


async fn handle_media_body(
    body: Body,
    max_bytes: u64,
    input_path: PathBuf, // with filename already in input_path
) -> Result<u64, std::io::Error> {
    
    let mut stream = body.into_data_stream();

    let mut total_bytes: u64 = 0;

    let mut file = utils::create_file(&input_path).await?;

    while let Some(stream_chunk) = stream.next().await {
        match stream_chunk {
            Ok(chunk) => {
                match total_bytes.checked_add(chunk.len() as u64) {
                    Some(new_total) if new_total <= max_bytes => {
                        total_bytes = new_total;

                        match file.write_all(&chunk).await {
                            Ok(_) => {},
                            Err(_) => {
                                utils::remove(&input_path).await?;
                                return Err(
                                    std::io::Error::other(
                                        "Couldn't write the chunk"
                                    )
                                );
                            }
                        }
                    },
                    _ => {
                        utils::remove(&input_path).await?;
                        return Err(
                            std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                "Size limit exceeded"
                            )
                        );
                    }
                }
            },
            Err(_) => {
                return Err(
                    std::io::Error::other(
                        "Couldn't stream the chunk"
                    )
                );
            },
        }
    }

    match file.flush().await {
        Ok(_) => {
            match file.sync_all().await {
                Ok(_) => {
                    if utils::is_empty(&input_path).await {
                        utils::remove(&input_path).await?;

                        return Err(
                            std::io::Error::other(
                                "File was found to be empty!"
                            )
                        );
                    }

                    Ok(total_bytes)
                },
                Err(_) => {
                    return Err(
                        std::io::Error::other(
                            "Couldn't properly write all bytes into disk"
                        )
                    )
                }
            }
        },
        Err(_) => {
            return Err(
                std::io::Error::other(
                    "Couldn't flush the bytes into file!"
                )
            );
        }
    }
}

