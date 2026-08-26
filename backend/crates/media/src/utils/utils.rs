use std::path::PathBuf;
use axum::http::StatusCode;

pub async fn create_dir(path: &PathBuf) -> Result<(), (StatusCode, String)>{
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;
    let filepath = path.display().to_string();
    match tokio::fs::create_dir_all(&path).await {
        Ok(()) => Ok(()),
        Err(_) => {
            // println!("{internal_server_error}: Couldn't create the required path '{filepath}'! \n");
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Couldn't create the required path '{filepath}'!\n")
            ))
        }
    }
}

pub async fn create_file(path: &PathBuf) -> Result<tokio::fs::File, (StatusCode, String)> {
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;
    let filepath = path.display().to_string();
    match tokio::fs::File::create(&path).await {
        Ok(f) => Ok(f),
        Err(_) => {
            // println!("{internal_server_error}: Couldn't create a directory in {filepath}");
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Couldn't create a directory in {filepath}")
            ))
        }
    }
}

pub async fn remove(path: &PathBuf) -> Result<(), (StatusCode, String)>{
    let filepath = path.display().to_string();
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;

    match tokio::fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(_) => {
            // println!("{internal_server_error}: Couldn't remove file '{filepath}'!\n");
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Couldn't remove file '{filepath}'!\n")
            ))
        }
    }
}

pub async fn rename(old_path: &PathBuf, new_path: &PathBuf) -> Result<(), (StatusCode, String)> {
    let bad_request = StatusCode::BAD_REQUEST;

    // To ensure empty or negative files aren't moved
    if is_empty(&old_path).await {
        let old = old_path.display().to_string();
        return Err((
            bad_request,
            format!("{bad_request}: File '{old}' is of invalid size!\n")
        ))
    }
        
    let internal_server_error = StatusCode::INTERNAL_SERVER_ERROR;
    let old_file_path = old_path.display().to_string();
    let new_file_path = new_path.display().to_string();
    match tokio::fs::rename(&old_path, &new_path).await {
        Ok(()) => Ok(()),
        Err(e) => {
            // println!("{internal_server_error}: Error '{e}' while renaming file '{old_file_path}' to '{new_file_path}'!\n");
            return Err((
                internal_server_error,
                format!("{internal_server_error}: Error '{e}' while renaming file '{old_file_path}' to '{new_file_path}'!\n")
            ))
        }
    }
}

pub async fn is_empty(path: &PathBuf) -> bool {
    // println!("Checking empty file right now");
    match tokio::fs::metadata(&path).await {
        Ok(bytes) if bytes.len() <= 0 => {
            // println!("File is empty");
            return true
        },
        _ => {
            // println!("File is not empty");
            return false
        }
    }
}
