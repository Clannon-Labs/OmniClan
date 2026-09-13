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

        let destination_path = MediaPaths::initialize()
            .await?
            .create_final_location(id)
            .await?;

        // println!("[READY MEDIA]: Destination path: {:?}", destination_path);
        
       for artifact in &mut self.artifacts {
           let old_path = artifact.path();

           // println!("[READY MEDIA]: Old path: {:?}", old_path);

           let filename = match old_path.file_name() {
                   Some(filename) => filename.display().to_string(),
                   None => "unknown".to_string()
            };
           
           let new_path = destination_path
               .join(filename);

           // println!("[READY MEDIA]: New path: {:?}", &new_path);
           
           utils::rename(&old_path, &new_path).await?;

           artifact.set_path(new_path);
       } 

       // println!("[READY MEDIA]: Artifacts: {:?}", &self.artifacts);
       
        Ok(
            FinalMedia {
                final_path: destination_path,
                artifacts: self.artifacts
            }
        )
    }
}
