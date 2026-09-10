
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::*;

pub(crate) trait MediaTrait {
    fn id(&self) -> &Uuid;
    fn created_at(&self) -> &DateTime<Utc>;
    fn updated_at(&self) -> &DateTime<Utc>;
    fn state(&self) -> &MediaState;

    // Its results after we probed it
    // 
    // Currently I return video if both video
    // and audio are available, cuz we can get
    // audio from a video but not vice versa
    // and having just video might indicate that 
    // either something failed or we are in the process
    // which is unusual,
    // but this logic might not be as good, so we gotta
    // change it later
    fn source_artifact(&self) -> Option<&Artifact> {
        let artifacts = match &self.state() {
            MediaState::Ready(ready) => &ready.artifacts,
            MediaState::Final(finalized) => &finalized.artifacts,
            _ => return None,
        };

        artifacts.iter().find(|artifact| {
            matches!(artifact, Artifact::Video(_))
        })
    }

    // I thought it will be Option<> because artifacts
    // would be returned only if the current state is
    // the followings
    fn artifacts(&self) -> Option<&[Artifact]> {
        match self.state() {
            MediaState::Ready(ready) => Some(&ready.artifacts),
            MediaState::Final(finalized) => Some(&finalized.artifacts),
            _ => None,
        }
    }

    fn path(&self) -> &Path {
        match self.state() {
            MediaState::Uploading(media) => &media.partial_path,
            MediaState::Uploaded(media) => &media.source_path,
            MediaState::Processing(media) => &media.source_path,
            MediaState::Ready(media) => &media.ready_path,
            MediaState::Final(media) => &media.final_path,
            _ => unreachable!()
        }
    }
    
    fn is_uploading(&self) -> bool {
        matches!(self.state(), MediaState::Uploading(_))
    }
    
    fn is_uploaded(&self) -> bool {
        matches!(self.state(), MediaState::Uploaded(_))
    } 

    fn is_processing(&self) -> bool {
        matches!(self.state(), MediaState::Processing(_))
    }
    
    fn is_ready(&self) -> bool {
        matches!(self.state(), MediaState::Ready(_))
    }
    
    fn is_final(&self) -> bool {
        matches!(self.state(), MediaState::Final(_))
    }
    
    fn is_failed(&self) -> bool {
        matches!(self.state(), MediaState::Failed(_))
    }    
}

impl MediaTrait for Media {
    fn id(&self) -> &Uuid {
        &self.id
    }

    fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    fn state(&self) -> &MediaState {
        &self.state
    }
}

impl Media {
    // Call the default path() method internally
    pub(crate) fn path(&self) -> &Path {
        <Self as MediaTrait>::path(&self)
    }
}