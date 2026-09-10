
use axum::{
    http::HeaderMap,
    body::Body,
};

use uuid::Uuid;
use super::super::{    
    UploadingMedia,
    UploadedMedia,
    
    MediaPaths,

    super::constants::MAX_UPLOAD_BYTES,
    super::UploadError,
};
// use super::super::super::{
//     handle::UploadRequest,
// };
use super::handle::handle_uploading;

impl UploadingMedia {
    pub(crate) async fn new(id: &Uuid) -> Result<Self, std::io::Error> {
        let partial_path = MediaPaths::initialize()
            .await?
            .create_staging_file(id)?;
        
        Ok(Self {
            partial_path: partial_path, // PathBuf
            uploaded_bytes: 0,
        })
    }

    pub(crate) async fn complete_upload(
        self,
        headers: HeaderMap,
        body: Body,
    ) -> Result<UploadedMedia, UploadError> {
                
        let uploaded_bytes = handle_uploading(
            &headers,
            body,
            MAX_UPLOAD_BYTES,
            self.partial_path.clone(),
        ).await
        .map_err(|err| UploadError::Io {
            source: err,
            path: Some(self.partial_path.clone())
        })?;

        Ok(UploadedMedia{
            source_path: self.partial_path,
            uploaded_bytes: uploaded_bytes,
        })
    }

}
