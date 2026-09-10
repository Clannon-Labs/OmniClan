
use std::path::Path;

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
    pub(crate) async fn process(
        self,
        source_path: &Path,
    ) -> Result<ReadyMedia, UploadError> {

        // Inspect the media/video via ffprobe 
        // and get result back 
        let probe = probe::probe_from_path(
            source_path,
        ).await?;
        
        if probe.has_video(){     
            let video_path = self.source_path
                .with_file_name(
                    constants::VIDEO_NAME
            );
            
            let audio_destination = self.source_path.clone()
                .with_file_name(
                    constants::AUDIO_NAME
            );

            let video = VideoArtifact::get_video_artifact(
                source_path,
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
            
            Ok(ReadyMedia {
                ready_path: self.source_path,
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
