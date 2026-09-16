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

      // To get the paths from the artifacts.
      // Why not just read all the files from
      // the directory and move?
      // Because artifacts are very important and we
      // can't just "assume" anything about them.
      // 
      // And we also gotta update their specific path.
      // And we can safely remove that directory after the
      // move has been completed.
       for artifact in &mut self.artifacts {
           let old_path = artifact.path();

           let filename = old_path.file_name()
               .map(|f|
                   f.display().to_string())
               .ok_or( // Return error instead of trying with name unknown
                   // which will never match because we haven't used unknown anywhere
                   UploadError::InvalidState {
                       reason: "Couldn't get the filename of the artifact!".to_string()
                   }
               )?;
           
           let new_path = destination_path
               .join(filename);
           
           utils::rename(&old_path, &new_path).await?;

           artifact.set_path(new_path);
       } 

      utils::remove(&self.ready_path).await?;
        
        Ok(
            FinalMedia {
                final_path: destination_path,
                artifacts: self.artifacts
            }
        )
    }
}
