use chrono::{DateTime, Utc};
use uuid::Uuid;

use axum::{
    http::{
        HeaderMap,
        // header,
    },
    body::Body,
};

use super::states::*;
use super::error::*;

#[derive(Debug)]
pub(crate) struct UploadRequest {
    pub(crate) headers: HeaderMap,
    pub(crate) body: Body,
}


/*
 * Summary of how data is flowing in the
 * handler function below:
 * 
 * We have multiple states (see ./states.rs),
 * and everytime we call the functions (eg: complete_upload(),
 * .process() etc), they give us the step next to them.
 * 
 * Like the .complete_upload() function from UploadMedia 
 * gives us ProcessingMedia, .process() from ProcessingMedia
 * gives use ReadyMedia and so on.
 */

pub(crate) async fn handle_upload(
    request: UploadRequest,
) -> Result<FinalMedia, UploadError> {

    // Initialize the unique id and set the created time
    let id = Uuid::now_v7();

    // extract the timestamp from id and convert it to 
    // DateTime to get almost identical time the id was created
    // 
    // If it failed, fallback to just creating time
    let now: DateTime<Utc> = match id.get_timestamp() {
        Some(stamp) => {
            let (seconds, nanoseconds) = stamp.to_unix();
            match DateTime::from_timestamp(seconds as i64, nanoseconds) {
                Some(time) => time,
                None => Utc::now()
            }
                
        },
        None => Utc::now()
    };
    
    // Now create a new UploadingMedia destination
    let upload = UploadMedia::new(&id).await?;    

    // Initialize media object with UploadingMedia state
    let mut media = Media {
        id,
        created_at: now,
        updated_at: now,
        state: MediaState::Upload(upload),
    };
    
    // uploading was moved into Media's state, so extract 
    // uploading again
    let upload = match media.state {
        MediaState::Upload(upload) => upload,
        _ => unreachable!(),
    };

    // Get the uploading struct and call the function
    let processing = upload
        .complete_upload(request.headers, request.body)
        .await?;

    // Change the new state and the updated_time
    // And repeat the same for all states.
    media.state = MediaState::Processing(processing);
    media.updated_at = Utc::now();

    media.write_manifest().await?;

    // uploaded was moved into media.state, so extract
    // it again
    let processing = match media.state {
        MediaState::Processing(processing) => processing,
        _ => unreachable!(),
    };

    // 
    // There's just one struct in the processing state
    // and it's ProcessingMedia.
    // 
    let ready = processing.clone()
        .process(&processing.path())
        .await?;
    
    media.state = MediaState::Ready(ready);
    media.updated_at = Utc::now();

    media.write_manifest().await?;
    
    // Extract processing again cuz it was moved
    let ready = match media.state {
        MediaState::Ready(ready) => ready,
        _ => unreachable!(),
    };

    let finalized_media = ready
        .finalize(&media.id)
        .await?;

    media.state = MediaState::Final(finalized_media);
    media.updated_at = Utc::now();

    media.write_manifest().await?;

    println!("[UPLOAD HANDLER]: Successfully processed the media!");
        
    let finalized_media = match media.state {
        MediaState::Final(finalized) => finalized,
        _ => unreachable!(),
    };
    
    Ok(finalized_media)
}
