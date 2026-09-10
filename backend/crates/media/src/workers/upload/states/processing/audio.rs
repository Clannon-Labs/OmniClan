use std::path::Path;

use super::super::AudioArtifact;

impl AudioArtifact {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}


