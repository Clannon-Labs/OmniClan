// This latest code was written on a phone and isn't tested,
// so there might be some mistakes/errors or bugs that need to be fixed.

// Also the logic might be wrong here, I think the old manifest should just be 
// deleted instead of moved because the manifest is gonna be created anyway in 
// the final directory 

use uuid::Uuid;
use tokio::fs;

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
      // And we also gotta update their specific path.
      // But if they succeded, we can "assume" that
      // the directory might have manifest there.
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

      // Now to get remaining files (just manifest.json for now)
      let mut entries = fs::read_dir(&self.ready_path).await
        .map_err(
          |e| UploadError::Io {
            source: e,
            path: Some(&self.ready_path.clone())
          }
        );

      while let Some(entry) = entries.next_entry().await
        .map_err(
          |e| UploadError::Io {
            source: e,
            path: Some(&self.ready_path.clone())
          }
        );
      {
        let current_path = entry.path();

        if current_path.is_file() {
          let filename = match current_path.file_name() {
            Some(name) => name,
            None => "manifest.json"
          };

          let new_path = destination_path.join(filename);
          utils::rename(&current_path, &new_path).await?;
        }
      }
        
        Ok(
            FinalMedia {
                final_path: destination_path,
                artifacts: self.artifacts
            }
        )
    }
}
