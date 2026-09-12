
use std::path::Path;
use tokio::process::Command;

use super::super::{
    VideoMetadata,
    VideoArtifact,

    AudioArtifact,
    AudioMetadata,
};
use super::probe::{self, ProbeOutput};
use super::super::super::UploadError;

impl VideoArtifact {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    
    pub(crate) async fn get_video_artifact(
        video_path: &Path,
        probe: &ProbeOutput,
    ) -> Result<Self, std::io::Error> {

        // println!("[VIDEO ARTIFACT]: About to get video artifact from {}", video_path.display());
        let video_stream = probe.video_streams().next()
            .ok_or_else (
                || std::io::Error::other(
                    "Couldn't get the video data from file!"
                )
            )?;
        // println!("[VIDEO ARTIFACT]: Got video stream: {:?}", video_stream);

        // println!("[VIDEO ARTIFACT]: Now trying to return the VideoArtifact");
        Ok(Self {
            path: video_path.to_path_buf(),
            metadata: VideoMetadata {
                format: probe.format.format_name.clone(),
                codec: video_stream.codec_name().to_string(),
                fps: video_stream.avg_fps()?,
                height: video_stream.height()?,
                width: video_stream.width()?,
                duration_ms: video_stream.duration_ms()?,
                size_bytes: probe.size_bytes()?,
                has_audio: probe.has_audio(),
            }
        })
    }
    
    pub(crate) async fn extract_audio(
        old_probe: &ProbeOutput,
        source_path: &Path,
        audio_destination: &Path,
    ) -> Result<AudioArtifact, UploadError> {

        // This check might be redundant here because it's caller
        // will check if video is available before calling it but
        // still.. better safe than sorry
        if !old_probe.has_video() {
            println!("[VIDEO ARTIFACT]: No video found to extract audio from in '{}'", source_path.display());
            return Err(
                UploadError::InvalidMedia {
                    reason: format!("No video found to extract audio from in '{source_path:?}'"),
                }
            )
        }
        
        if !old_probe.has_audio() {
            println!("[VIDEO ARTIFACT]: No audio found to extract in '{}'", source_path.display());
            return Err(
                UploadError::InvalidMedia {
                    reason: format!("No audio found to extract in '{source_path:?}'"),
                }
            )
        }
        
        let output = match Command::new("ffmpeg")
            .arg("-i")
            .arg(source_path)
            .args([
                "-vn",
                "-c:a",
                "copy",
            ])
            .arg(audio_destination) // we must need a filename for audio
            .output()
            .await {
                Ok(output) => output,
                Err(err) => {
                    return Err(
                        UploadError::Io{
                            source: err,
                            path: Some(source_path.to_path_buf()),
                        }
                    )
                }
            };

        let stderr = String::from_utf8_lossy(
            &output.stderr
        );

        if !output.status.success() {
            return Err(
                UploadError::Io {
                    source: std::io::Error::other(stderr),
                    path: Some(source_path.to_path_buf()),
                }
            )
        }

        let probe = probe::probe_from_path(
            audio_destination
        ).await
        .map_err(|err| UploadError::Io {
            source: err,
            path: Some(audio_destination.to_path_buf()),
        })?;
        
        let audio = probe.audio_streams().next()
            .ok_or_else(
                || UploadError::InvalidMedia {
                    reason: "No audio stream found!".to_string()
                }
                )?;
        
        Ok(
            AudioArtifact {
                path: audio_destination.to_path_buf(),
                metadata: AudioMetadata {
                    format: probe.format.format_name.clone(),
                    codec: audio.codec_name().to_string(),
                    size_bytes: probe.size_bytes()?,
                    duration_ms: audio.duration_ms()?,
                    sample_rate: audio.sample_rate()?,
                    channels: audio.channels()?,
                }
            }
        )
    }

}

