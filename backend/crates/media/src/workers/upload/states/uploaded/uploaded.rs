use std::path::PathBuf;
use uuid::Uuid;
use super::super::{
    UploadedMedia,     
    ProcessingMedia,
    
    MediaPaths,
    
    super::utils,

    super::UploadError,
};


impl UploadedMedia {
    pub(crate) async fn get_processing(
        self,
    ) -> Result<ProcessingMedia, UploadError> {

        // Currently I haven't thought of what to do once
        Ok(ProcessingMedia {
            source_path: self.source_path,
        })
    }
}
