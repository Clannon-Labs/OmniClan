use std::path::{Path, PathBuf};
use uuid::Uuid;
use super::constants::*;
use super::utils;

pub(crate) struct MediaPaths;

impl MediaPaths {
    fn get_temporary_directory() -> PathBuf {
        Path::new(UPLOAD_LOCATION).join(TEMPORARY_LOCATION)
    }

    fn get_final_directory() -> PathBuf {
        Path::new(UPLOAD_LOCATION).join(FINAL_LOCATION)
    }

    async fn create_temporary_location(
        id: &Uuid,
    ) -> Result<PathBuf, std::io::Error> {
        let temp_location = Self::get_temporary_directory()
            .join(id.to_string());

        utils::create_dir(&temp_location).await?;

        Ok(temp_location)
    }

    pub(crate) async fn get_temporary_file(
        id: &Uuid
    ) -> Result<PathBuf, std::io::Error> {
        let temp_loc = Self::create_temporary_location(id).await?;
        Ok(temp_loc.join(TEMPORARY_FILE_NAME))
    }
    
    pub(crate) async fn create_final_location(
        id: &Uuid
    ) -> Result<PathBuf, std::io::Error> {
        let final_location = Self::get_final_directory()
            .join(id.to_string());

        utils::create_dir(&final_location).await?;

        Ok(final_location)
    }
}
