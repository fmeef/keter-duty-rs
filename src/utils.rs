use anyhow::Context as _;
use anyhow::Result;
use directories::ProjectDirs;
use include_dir::include_dir;
use include_dir::{Dir, DirEntry};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
/// Default sandbox templates
pub(crate) static SANDBOX_DIR: Dir<'_> = include_dir!("sb");

/// Gets the library path including the user's home directory
pub(crate) fn get_library_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "KeterDuty").context("Failed to determine project paths")?;

    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir).context("Failed to ensure data dir exists")?;
    Ok(data_dir.to_owned())
}

/// Runs sandbox-exec process with rendered template
pub(crate) fn sandbox_exec(rendered: &str, path: &Path, args: &[String]) -> Result<()> {
    Command::new("sandbox-exec")
        .arg("-p")
        .arg(rendered)
        .arg(path)
        .args(args)
        .output()?;
    Ok(())
}

/// Check if the library directory contains a given name
pub(crate) fn library_path_contains(name: &str) -> Result<bool> {
    let p = get_library_path()?;
    for dir in p.read_dir()? {
        if let Ok(dir) = dir {
            if dir.file_name() == name {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Copy over default templates to config directory.
/// Skips existing files, unlike [Dir::extract].
pub(crate) fn populate_sb_tree<T: AsRef<Path>>(dir: &Dir<'_>, base_path: &T) -> Result<()> {
    let base_path = base_path.as_ref();
    for entry in dir.entries() {
        let path = base_path.join(entry.path());
        match entry {
            DirEntry::Dir(d) => {
                fs::create_dir_all(&path)?;
                populate_sb_tree(d, &base_path)?;
            }
            DirEntry::File(file) => {
                if !path.exists() {
                    fs::write(path, file.contents())?;
                }
            }
        }
    }
    Ok(())
}
