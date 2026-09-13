use std::path::Path;
use tokio::fs::{self, File};

pub(crate) async fn create_dir(path: &Path) -> Result<(), std::io::Error>{

    if path.is_file() {
        return Err(
            std::io::Error::other(
                format!("{path:?} is not a directory!")
            )
        )
    }
    
    match fs::create_dir_all(&path).await {
        Ok(()) => Ok(()),
        Err(_) => {
            return Err(
                std::io::Error::other(
                    format!("Couldn't create the directory: {path:?}")
                )
            )
        }
    }
}

pub(crate) async fn create_file(path: &Path) -> Result<tokio::fs::File, std::io::Error> {
    // println!("[UPLOADING MEDIA UTILS]: About to create a file in {}", path.display());

    if let Some(parent) = path.parent() {
        create_dir(parent).await?
    }
    
    match File::create(&path).await {
        Ok(f) => Ok(f),
        Err(_) => {
            // println!("[UPLOADING MEDIA UTILS]: Failed to create a file in {}", path.display());
            return Err(
                std::io::Error::other(
                    format!("Couldn't create a directory in {path:?}!")
                )
            )
        }
    }
}

pub(crate) async fn remove(path: &Path) -> Result<(), std::io::Error>{
    let filepath = path.display().to_string();

    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(_) => {
            return Err(
                std::io::Error::other(
                    format!("Couldn't remove file '{filepath}'!")
                )
            )
        }
    }
}

// We have yet to add a check to ensure both paths have
// a file or directory at the end, or else it would throw
// an error
pub(crate) async fn rename(old_path: &Path, new_path: &Path) -> Result<(), std::io::Error> {
    // To ensure empty or negative files aren't moved
    if is_empty(&old_path).await {
        let old = old_path.display().to_string();
        return Err(
            std::io::Error::other(
                format!("File '{old}' is of invalid size!")
            )
        )
    }
        
    let old_file_path = old_path.display().to_string();
    let new_file_path = new_path.display().to_string();
    match fs::rename(&old_path, &new_path).await {
        Ok(()) if old_path.exists() => {
            remove(&old_path).await?;
            return Ok(())
        },
        Ok(()) => Ok(()),
        Err(e) => {
            return Err(
                std::io::Error::other(
                    format!("Error '{e}' while renaming file '{old_file_path}' to '{new_file_path}'!")
                )
            )
        }
    }
}

pub(crate) async fn is_empty(path: &Path) -> bool {
    match fs::metadata(&path).await {
        Ok(bytes) if bytes.len() <= 0 => {
            return true
        },
        _ => {
            return false
        }
    }
}

