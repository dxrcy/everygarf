use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::NaiveDate;

use crate::dates::date_from_filename;

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

pub fn create_dir(path: &Path, remove_existing: bool) -> Result<(), String> {
    if path.exists() {
        if path.is_file() {
            return Err(format!("target directory is a file"));
        }
        if remove_existing {
            fs::remove_dir_all(path).map_err(|err| format!("remove directory - {:#?}", err))?;
        } else {
            return Ok(());
        }
    }
    fs::create_dir_all(path).map_err(|err| format!("create directory - {:#?}", err))?;
    Ok(())
}

pub fn get_existing_dates(folder: &Path) -> Result<Vec<NaiveDate>, String> {
    let mut dates = Vec::new();

    let children = fs::read_dir(folder).map_err(|err| format!("read directory - {:#?}", err))?;
    for child in children.flatten() {
        if !child.path().is_file() {
            continue;
        }
        let filename = child.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };

        let Some(date) = date_from_filename(filename) else {
            continue;
        };
        dates.push(date);
    }

    Ok(dates)
}
