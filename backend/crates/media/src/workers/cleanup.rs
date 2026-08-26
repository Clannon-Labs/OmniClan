use axum::http::StatusCode;
use std::path::PathBuf;
use tokio::fs;

use crate::constants::{
    PARTIAL_FILE,
    READY_FILE,
    FINAL_FILE,
    TEMP_LOCATION,
    FINAL_LOCATION,
};

/*
Plan is to goto uploads directory and see the temp
and check is any .ready file is available, if it is,
try to move it into media/ and name it .final (basic recovery)
and delete the .part files because they are not processed or
validated (basic validation)
*/

pub async fn inspect_and_cleanup() -> (StatusCode, String) {
    let mut entries = match fs::read_dir(TEMP_LOCATION).await {
        Ok(entries) => entries,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Couldn't read temp directory: {e}"),
            );
        }
    };

    // we need a loop to lookup the files
    // it can't happen in one shot!!
    while let Some(entry) = match entries.next_entry().await {
        Ok(entry) => entry,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Couldn't read directory entry: {e}"),
            );
        }
    } {
        let path = entry.path();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str());

        match extension {
            Some(PARTIAL_FILE) => {
                if let Err(e) = fs::remove_file(&path).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Couldn't remove {path:?}: {e}"),
                    );
                }
            }

            Some(READY_FILE) => {
                let mut new_path = PathBuf::from(FINAL_LOCATION);

                let filename = entry.file_name();
                let filename_path = PathBuf::from(&filename);

                // Build destination using original filename
                new_path.push(filename_path);

                // Now set the extension from .ready to .final
                new_path.set_extension(FINAL_FILE);

                if let Err(e) = fs::rename(&path, &new_path).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Couldn't move {path:?} -> {new_path:?}: {e}"),
                    );
                }
            }

            Some(FINAL_FILE) => {
                let mut new_path = PathBuf::from(FINAL_LOCATION);
                new_path.push(entry.file_name());

                if let Err(e) = fs::rename(&path, &new_path).await {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Couldn't move {path:?} -> {new_path:?}: {e}"),
                    );
                }
            }

            Some(_) => {
                // Unknown file type, so leave it alone
                // for now
            }

            None => {
                // File without extension, so leave it alone.
            }
        }
    }

    (
        StatusCode::OK,
        "Cleanup completed".to_string(),
    )
}
