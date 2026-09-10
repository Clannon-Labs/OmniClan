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
    pub(crate) async fn start_processing(
        self,
        id: &Uuid,
    ) -> Result<ProcessingMedia, UploadError> {
           
        let paths = MediaPaths::initialize().await?;
        
        let new_location = paths
            .create_processing_location(id)
            .await
            .map_err(|err| UploadError::Io {
                source: err,
                path: Some(self.source_path.clone())
            })?;

        let mut new_file = PathBuf::new();
        
        if let Some(filename) = &self
            .source_path
            .file_name() {
                let new = new_location.join(filename);
                new_file.push(&new);
                
                utils::rename(
                    &self.source_path,
                    &new
                ).await
                .map_err(|err| UploadError::Io {
                    source: err,
                    path: Some(new),
                })?;
            }

        Ok(ProcessingMedia {
            source_path: new_file,
        })
    }
}
