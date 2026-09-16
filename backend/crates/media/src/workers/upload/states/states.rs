
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::path::{PathBuf, Path};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde( rename_all = "snake_case" )]
pub(crate) enum MediaState {
    Upload(UploadMedia),
    // Uploaded(UploadedMedia),
    Processing(ProcessingMedia),
    Ready(ReadyMedia),
    Final(FinalMedia),
    Failed(FailedMedia),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Media {
    pub(crate) id: Uuid,
    pub(crate) created_at: DateTime<Utc>,
    pub(crate) updated_at: DateTime<Utc>,
    pub(crate) state: MediaState,
}

// When user first uploads the media;
// the streaming process
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct UploadMedia {
    pub(crate) partial_path: PathBuf,
    pub(crate) uploaded_bytes: u64,
}

// Now processing the video and trying to extract audio if available
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ProcessingMedia {
    pub(crate) source_path: PathBuf,
    pub(crate) source_bytes: u64,
}

// Video is ready to be commited to permanent storage and
// if available, audio too
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ReadyMedia {
    pub(crate) ready_path: PathBuf,
    pub(crate) artifacts: Vec<Artifact>,
}

// Basic validation done and video is maybe in a good form
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FinalMedia {
    pub(crate) final_path: PathBuf,
    pub(crate) artifacts: Vec<Artifact>,
}

// Somewhere in the process, the processing failed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FailedMedia {
    pub(crate) failed_stage: ProcessingStage,
    pub(crate) error: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) enum ProcessingStage {
     // When it was first being uploaded
    Upload,
    
    // When we were inspecting the media to know what it
    // is actually
    Inspect, 

    // Extraction of the audio
    AudioExtraction,

    // When we were making sure the media is good and
    // validating if it has anything we dont want to 
    // accept
    Validation, 
    
    // Final checks to move to permanent storage
    Finalization,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde( rename_all = "snake_case" )]
pub(crate) enum Artifact {
    Video(VideoArtifact),
    Audio(AudioArtifact),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct VideoArtifact {
    pub(crate) path: PathBuf,
    pub(crate) metadata: VideoMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AudioArtifact {
    pub(crate) path: PathBuf,
    pub(crate) metadata: AudioMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct VideoMetadata {
    pub(crate) format: String, // container/codec_name(webm/mp4)
    pub(crate) codec: String, // video
    pub(crate) duration_ms: u64,

    pub(crate) fps: FrameRate,
    
    pub(crate) width: u32,
    pub(crate) height: u32,

    pub(crate) size_bytes: u64,
    
    pub(crate) has_audio: bool,
    // pub(crate) audio_codec_type: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AudioMetadata {
    pub(crate) format: String, // container/codec_name(mp3/opus etc)
    pub(crate) codec: String, // audio

    pub(crate) size_bytes: u64,
    
    pub(crate) duration_ms: u64,
    pub(crate) sample_rate: u32,
    pub(crate) channels: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde( rename_all = "snake_case" )]
pub(crate) struct FrameRate {
    pub(crate) numerator: u32,
    pub(crate) denominator: u32,
}

impl Artifact {
    pub(crate) fn path(&self) -> &Path {
        match self {
            Self::Video(video) => &video.path,
            Self::Audio(audio) => &audio.path,
        }
    }

    pub(crate) fn set_path(&mut self, path: PathBuf) -> Option<()> {
        match self {
            Self::Video(video) => {
                video.path = path;
                Some(())
            },
            Self::Audio(audio) => {
                audio.path = path;
                Some(())
            },
        }
    }
}
