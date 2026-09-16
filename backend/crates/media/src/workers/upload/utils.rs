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

    if let Some(parent) = path.parent() {
        create_dir(parent).await?
    }
    
    match File::create(&path).await {
        Ok(f) => Ok(f),
        Err(_) => {
            return Err(
                std::io::Error::other(
                    format!("Couldn't create a directory in {path:?}!")
                )
            )
        }
    }
}

pub(crate) async fn remove(path: &Path) -> Result<(), std::io::Error>{
    if path.is_file() {
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(_) => {
                return Err(
                    std::io::Error::other(
                        format!("Couldn't remove file '{path:?}'!")
                    )
                )
            }
        }
    } else {
        match fs::remove_dir_all(&path).await {
            Ok(()) => Ok(()),
            Err(_) => {
                return Err(
                    std::io::Error::other(
                        format!("Couldn't remove directory '{path:?}'!")
                    )
                )
            }
        }
    }
    
}

// We have yet to add a check to ensure both paths have
// a file or directory at the end, or else it would throw
// an error
pub(crate) async fn rename(old_path: &Path, new_path: &Path) -> Result<(), std::io::Error> {
    // To ensure empty or negative files aren't moved
    if is_empty(&old_path).await {
        return Err(
            std::io::Error::other(
                format!("File '{old_path:?}' is of invalid size!")
            )
        )
    }
        
    match fs::rename(&old_path, &new_path).await {
        Ok(()) if old_path.exists() => {
            remove(&old_path).await?;
            return Ok(())
        },
        Ok(()) => Ok(()),
        Err(e) => {
            return Err(
                std::io::Error::other(
                    format!("Error '{e}' while renaming file '{old_path:?}' to '{new_path:?}'!")
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

