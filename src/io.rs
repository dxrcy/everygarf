use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

pub fn get_folder_path(folder: Option<String>) -> Result<PathBuf, String> {
    let folder = folder.map(|folder| Path::new(&folder).to_path_buf());

    if let Some(folder) = folder {
        return Ok(folder);
    }
    if let Some(folder) = get_generic_parent_folder() {
        return Ok(folder.join(&Path::new("garfield")));
    }

    Err(format!(
        "Cannot automatically find appropriate folder location. Please enter folder manually"
    ))
}

fn get_generic_parent_folder() -> Option<PathBuf> {
    use dirs_next::*;
    picture_dir().or_else(document_dir).or_else(home_dir)
}

pub fn create_target_dir(path: &Path, remove_existing: bool) -> io::Result<()> {
    if path.exists() {
        if path.is_file() {
            return Err(io::Error::from(io::ErrorKind::AlreadyExists));
        }
        if remove_existing {
            fs::remove_dir_all(path)?;
        } else {
            return Ok(());
        }
    }
    fs::create_dir_all(path)
}

pub fn get_child_filenames(folder: &Path) -> io::Result<impl Iterator<Item = OsString>> {
    Ok(fs::read_dir(folder)?
        .flatten()
        .filter(|child| child.path().is_file())
        .map(|child| child.file_name()))
}
