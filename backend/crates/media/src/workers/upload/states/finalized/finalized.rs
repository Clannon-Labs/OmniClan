
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::super::{
    ReadyMedia,
    FinalizedMedia,

    Artifact,

    manifest::Manifest,
    
    super::{
        UploadError,
        utils,
    },
};
use super::manifest::Manifest;


impl FinalizedMedia {

    pub(crate) async fn finalize_manifest(
    ) -> Result<Self, UploadError> {

        let manifest = Manifest::write(
            &ready_media
        ).await?;

        
    }
}