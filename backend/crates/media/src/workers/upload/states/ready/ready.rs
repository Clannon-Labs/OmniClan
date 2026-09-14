use uuid::Uuid;

use super::super::{
    ReadyMedia,
    FinalMedia,

    MediaPaths,

    super::UploadError,
    super::utils,
};

impl ReadyMedia {
    pub(crate) async fn finalize(
        mut self,
        id: &Uuid,
    ) -> Result<FinalMedia, UploadError> {

        let destination_path = MediaPaths::create_final_location(id).await?;
        
       for artifact in &mut self.artifacts {
           let old_path = artifact.path();

           let filename = match old_path.file_name() {
                   Some(filename) => filename.display().to_string(),
                   None => "unknown".to_string()
            };
           
           let new_path = destination_path
               .join(filename);
           
           utils::rename(&old_path, &new_path).await?;

           artifact.set_path(new_path);
       } 

        Ok(
            FinalMedia {
                final_path: destination_path,
                artifacts: self.artifacts
            }
        )
    }
}
