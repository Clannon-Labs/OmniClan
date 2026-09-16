
use std::path::Path;
use super::super::super::utils;

use super::super::{
    ProcessingMedia,
    ReadyMedia,
    
    Artifact,

    super::{
        UploadError,
        constants,
    },
};
use super::{
    probe,
    super::VideoArtifact,
    // super::AudioArtifact,
};


impl ProcessingMedia {
    pub(crate) fn path(&self) -> &Path {
        &self.source_path
    }
    
    pub(crate) async fn process(
        self,
        source_path: &Path
    ) -> Result<ReadyMedia, UploadError> {

        // Inspect the media/video via ffprobe 
        // and get result back 
        let probe = probe::probe_from_path(
            source_path,
        ).await?;
        
        if probe.has_video(){     
            let format = probe.format.format_name()?;
            
            let video_path = self.source_path
                .with_file_name(
                    constants::VIDEO_NAME
            ).with_extension(&format);

            utils::rename(&self.source_path, &video_path).await?;
            
            let audio_destination = self.source_path.clone()
                .with_file_name(
                    constants::AUDIO_NAME
            ).with_extension(&format);
            
            let video = VideoArtifact::get_video_artifact(
                &video_path,
                &probe
            ).await
            .map_err(
                |e| UploadError::Io {
                    source: e,
                    path: Some(source_path.to_path_buf()),
                }
            )?;
            
            let audio = VideoArtifact::extract_audio(
                &probe,
                &video_path,
                &audio_destination
            ).await
            .map_err(|err| UploadError::Io {
                source: std::io::Error::other(err),
                path: Some(audio_destination.to_path_buf())
            })?;

            // To get the directory of the final/ready path instead of path to a file
            let mut ready_path = self.source_path.clone();
            ready_path.pop();
                        
            Ok(ReadyMedia {
                ready_path: ready_path,
                artifacts: vec![
                    Artifact::Video(video),
                    Artifact::Audio(audio),
                ],
            })
        } else {
            return Err(
                UploadError::InvalidMedia {
                    reason: "No media other than video is currently accepted!".to_string(),
                }
            )
        }
    }
}
