use std::path::{Path, PathBuf};
// use tokio::fs;
use uuid::Uuid;
use super::constants::*;
use super::utils;

pub(crate) struct MediaPaths {
    root: PathBuf,
}

impl MediaPaths {
    pub(crate) async fn initialize() -> Result<Self, std::io::Error> {
        // println!("[MEDIA PATHS]: Initializing paths...");
        let paths = Self {
            root: PathBuf::from(UPLOAD_LOCATION),
        };

        // println!("[MEDIA PATHS]: About to call staging creator");
        utils::create_dir(&paths.staging()).await?;
        // println!("[MEDIA PATHS]: About to call processing creator");
        utils::create_dir(&paths.processing()).await?;
        // println!("[MEDIA PATHS]: About to call failed creator");
        utils::create_dir(&paths.failed()).await?;
        // println!("[MEDIA PATHS]: About to call final location creator");
        utils::create_dir(&paths.final_location()).await?;

        // println!("[MEDIA PATHS]: Paths initialized successfully");
        
        Ok(paths)
    }
    
    pub(crate) fn create_staging_file(
        &self,
        id: &Uuid)-> Result<PathBuf, std::io::Error> {
        
        let staging = self
            .staging()
            .join(format!("{id}"))
            .join(TEMPORARY_FILE_NAME);

        // println!("[MEDIA PATHS]: Returning {} from create_staging_file", staging.display());
        
        Ok(staging)
    }

    // pub(crate) async fn create_staging_file(
    //     &self,
    //     id: &Uuid,
    // ) -> Result<PathBuf, std::io::Error> {
    //     let file = self.create_staging_location(id)
    //         .await?
    //         .join(TEMPORARY_FILE_NAME);

    //     Ok(file)
    // }
    

    pub(crate) async fn create_processing_location(
        &self,
        id: &Uuid)-> Result<PathBuf, std::io::Error> {

        let processing = self.processing()
            .join(format!("{id}"));

        utils::create_dir(&processing).await?;

        Ok(processing)
    }

    // pub(crate) async fn get_processing_filename(
    //     &self,
    //     id: &Uuid
    // ) -> Result<PathBuf, std::io::Error> {
    //     let processing = self
    //         .create_processing_location(id)
    //         .await?
    //         .join(TEMPORARY_FILE_NAME);

    //     Ok(processing)
    // }

    pub(crate) async fn create_final_location(
        &self,
        id: &Uuid,
    ) -> Result<PathBuf, std::io::Error> {
        let final_path = self.final_location()
            .join(format!("{id}"));

        // println!("[MEDIA PATHS]: About to create final location");
        utils::create_dir(&final_path).await?;

        Ok(final_path)
    }
    
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn staging(&self) -> PathBuf {
        self.root.join(STAGING_NAME)
    }
    
    pub(crate) fn processing(&self) -> PathBuf {
        self.root.join(PROCESSING_NAME)
    }

    pub(crate) fn failed(&self) -> PathBuf {
        self.root.join(FAILED_NAME)
    }

    pub(crate) fn final_location(&self) -> PathBuf {
        self.root.join(FINAL_NAME)
    }
    
}
