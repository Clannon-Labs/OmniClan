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

        let final_path = MediaPaths::initialize()
            .await?
            .create_final_location(id)
            .await?;
        
       for artifact in &mut self.artifacts {
           let old_path = artifact.path().to_path_buf();

           let new_path = final_path
               .join(
                   old_path
                   .with_extension(&artifact.codec())
               );

           utils::rename(&old_path, &new_path).await?;

           artifact.set_path(new_path);
       } 
       
        Ok(
            FinalMedia {
                final_path: final_path,
                artifacts: self.artifacts
            }
        )
    }
}
