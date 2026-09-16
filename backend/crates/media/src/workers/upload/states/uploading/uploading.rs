
use axum::{
    http::HeaderMap,
    body::Body,
};

use uuid::Uuid;
use super::super::{    
    UploadMedia,
    ProcessingMedia,
    
    MediaPaths,

    super::constants::MAX_UPLOAD_BYTES,
    super::UploadError,
};
// use super::super::super::{
//     handle::UploadRequest,
// };
use super::handle::handle_uploading;

impl UploadMedia {
    pub(crate) async fn new(
        id: &Uuid,
    ) -> Result<Self, UploadError> {
        
        let partial_path = MediaPaths::get_temporary_file(id)
            .await?;

        // println!("\n[UPLOAD]: Created temporary media at: {:?}", &partial_path);
        
        Ok(Self {
            partial_path: partial_path, // PathBuf
            uploaded_bytes: 0,
        })
    }

    pub(crate) async fn complete_upload(
        self,
        headers: HeaderMap,
        body: Body,
    ) -> Result<ProcessingMedia, UploadError> {

        // println!("[UPLOAD]: Partial path: {:?}", &self.partial_path);
        
        let source_bytes = handle_uploading(
            &headers,
            body,
            MAX_UPLOAD_BYTES,
            &self.partial_path,
        ).await
        .map_err(|err| UploadError::Io {
            source: err,
            path: Some(self.partial_path.clone())
        })?;

        // println!("\n[UPLOAD]: Source path: {:?}\tSource bytes: {}", &self.partial_path, source_bytes);
        
        Ok(ProcessingMedia{
            source_path: self.partial_path,
            source_bytes: source_bytes,
        })
    }

}
