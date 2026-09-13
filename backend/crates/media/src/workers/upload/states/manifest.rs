
use serde::{Serialize, Deserialize};
use serde_json;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::DateTime;
use tokio::{
    fs,
    io::AsyncWriteExt,
};

use super::{
    Media,
    traits::MediaTrait,
    MediaState,  
    Artifact,
  
    super::UploadError,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct Manifest {
    pub(crate) id: Uuid,
    pub(crate) state: MediaState,
    // we could define a separate struct for Source
    // but it's not quite worth it..
    // so we can just store the Artifact struct as source too
    pub(crate) source: Option<Artifact>,
    pub(crate) artifacts: Option<Vec<Artifact>>,
    pub(crate) created_at: DateTime<chrono::Utc>,
    pub(crate) updated_at: DateTime<chrono::Utc>,
    pub(crate) error: Option<String>,
}


//
// The result we got after we probed the original
// media we had gotten.
// For example, the video(and not the extracted audio)
// if user had uploaded a video
// 
#[derive(Debug, Serialize, Deserialize)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct Source {
    // The filename uploaded by user could
    // be dangerous, so unless needed later,
    // we dont have to use it
    // pub(crate) filename: String,
    pub(crate) path: PathBuf,
    pub(crate) size_bytes: u64,
    pub(crate) container: String,
    pub(crate) media_type: Artifact,
}

impl Manifest {
    // Later change the UploadError to WriteError
    pub(crate) async fn write(
        media: &Media
    ) -> Result<(), UploadError> {

        let manifest = Self {
            id: *media.id(),
            state: media.state().clone(),
            source: media.source_artifact().cloned(),
            artifacts: media.artifacts().map(|a| a.to_vec()),
            created_at: media.created_at().clone(),
            updated_at: media.updated_at().clone(),
            error: None,
        }; 
        // Figure out a way to get the errors..
        // maybe take it as an argument??

        let path = media.path();

        // println!("\n[MANIFEST]: Writing manifest to: {:?}", &path);
        
        let bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(
                |error| UploadError::Io {
                    source: std::io::Error::new(
                        std::io::ErrorKind::Other,
                        error
                    ),
                    // use the path of the json here
                    path: Some(path.to_path_buf()),
                }
            )?;

        let filename = if path.is_file() {
            path.with_file_name("manifest.json")
        } else {
            path.join("manifest.json")
        };
        
        let mut file = match fs::File::create(filename)
            .await {
                Ok(f) => f,
                Err(e) => {
                    // println!("[MANIFEST]: Error occured while creating manifest file on {}", path.display());
                    return Err(
                        UploadError::Io {
                            source: e,
                            path: Some(path.to_path_buf())
                        }
                    );
                }
            };
        
        file.write_all(&bytes).await?;

        // Ensure the bytes are completely flushed
        // out from memory and onto OS page cache
        file.flush().await?;

        // Force the OS to write them into the disk
        // and/or flush out its cache immediately
        // into the disk
        file.sync_all().await?;
        
        Ok(())
    }
}

