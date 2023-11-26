use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn get_folder_path(folder: Option<String>) -> Result<PathBuf, String> {
    let folder = folder.map(|folder| Path::new(&folder).to_path_buf());

    let folder = match folder {
        Some(folder) => folder,
        None => match get_auto_parent_folder() {
            Some(folder) => folder.join(&Path::new("garfield")),
            None => {
                return Err(format!(
                    "Cannot automatically find appropriate folder location. Please enter folder manually"
                ))
            }
        },
    };

    if !Path::new(&folder).exists() {
        fs::create_dir(&folder)
            .map_err(|err| format!("Failed to create output folder - {err:?}"))?;
    }

    Ok(folder.to_path_buf())
}

fn get_auto_parent_folder() -> Option<PathBuf> {
    let dir = if let Some(dir) = dirs_next::picture_dir() {
        dir
    } else if let Some(dir) = dirs_next::document_dir() {
        dir
    } else if let Some(dir) = dirs_next::home_dir() {
        dir
    } else {
        return None;
    };

    Some(dir)
}

pub fn create_empty_target_directory(path: &Path) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Err(format!("target directory is a file"));
        }
        fs::remove_dir_all(path).map_err(|err| format!("remove directory - {:#?}", err))?;
    }
    fs::create_dir_all(path).map_err(|err| format!("create directory - {:#?}", err))?;
    Ok(())
}
