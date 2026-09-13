use std::path::{Path, PathBuf};
use uuid::Uuid;
// use tokio::fs;
use super::constants::*;
use super::utils;

pub(crate) struct MediaPaths;

impl MediaPaths {
    // async fn initialize() -> Result<(), std::io::Error> {
    //     utils::create_dir(&Self::get_temporary_directory()).await?;
    //     utils::create_dir(&Self::get_failed_directory()).await?;
    //     utils::create_dir(&Self::get_final_directory()).await?;
    //     Ok(())
    // }

    // async fn create_dir_from_const(dir_str: &'static str) -> Result<&'static Path, std::io::Error> {
    //     let path = Path::new(dir_str);
    //     utils::create_dir(&path).await?;
    //     Ok(path)
    // }
    
    fn get_temporary_directory() -> PathBuf {
        Path::new(UPLOAD_LOCATION).join(TEMPORARY_LOCATION)
    }

    fn get_failed_directory() -> PathBuf {
        Path::new(UPLOAD_LOCATION).join(FAILED_LOCATION)
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

    pub(crate) async fn create_failed_location(
        id: &Uuid
    ) -> Result<PathBuf, std::io::Error> {
        let failed_location = Self::get_failed_directory()
            .join(id.to_string());

        utils::create_dir(&failed_location).await?;

        Ok(failed_location)
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
