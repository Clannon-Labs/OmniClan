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
use super::states::manifest::Manifest;

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
 * start_processing() etc), they give us another state.
 * 
 * Like the .complete_upload() function from UploadingMedia 
 * gives us UploadedMedia, .start_processing() from UploadedMedia
 * gives use ProcessingMedia and so on.
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
    
    // Now create a new UploadignMedia destination
    let uploading = UploadingMedia::new(&id).await?;

    // Initialize media object with UploadingMedia state
    let mut media = Media {
        id,
        created_at: now,
        updated_at: now,
        state: MediaState::Uploading(uploading),
    };

    Manifest::write(&media).await?;
    
    // uploading was moved into Media's state, so extract 
    // uploading again
    let uploading = match media.state {
        MediaState::Uploading(uploading) => uploading,
        _ => unreachable!(),
    };

    // Get the uploading struct and call the function
    let uploaded = uploading
        .complete_upload(request.headers, request.body)
        .await?;

    // Change the new state and the updated_time
    // And repeat the same for all states.
    media.state = MediaState::Uploaded(uploaded);
    media.updated_at = Utc::now();

    Manifest::write(&media).await?;

    // uploaded was moved into media.state, so extract
    // it again
    let uploaded = match media.state {
        MediaState::Uploaded(uploaded) => uploaded,
        _ => unreachable!(),
    };

    // 
    // There's just one struct in the processing state
    // and it's ProcessingMedia.
    // 
    // The function in the UploadingMedia is called start_processing()
    // because it takes the first step which later enables processing
    // 
    // And it's just changing the location of media from partial
    // uploading location to processing location, but in future,
    // we could add other checks too without breaking anything..
    // 
    // Until we take the same params, return same stuff and
    //  change the directory properly.
    // 
    let processing = uploaded
        .start_processing(&media.id)
        .await?;
    
    media.state = MediaState::Processing(processing);
    media.updated_at = Utc::now();

    Manifest::write(&media).await?;
    
    // Extract processing again cuz it was moved
    let processing = match media.state {
        MediaState::Processing(ref processing) => processing,
        _ => unreachable!(),
    };

    let ready = processing.clone()
        .process(&media.path())
        .await?;

    media.state = MediaState::Ready(ready);
    media.updated_at = Utc::now();

    Manifest::write(&media).await?;
    
    let ready = match media.state {
        MediaState::Ready(ready) => ready,
        _ => unreachable!(),
    };
 
    let finalized_media = ready
        .finalize(&media.id)
        .await?;

    media.state = MediaState::Final(finalized_media);
    media.updated_at = Utc::now();

    Manifest::write(&media).await?;
    
    let finalized_media = match media.state {
        MediaState::Final(finalized) => finalized,
        _ => unreachable!(),
    };
    
    Ok(finalized_media)
}
