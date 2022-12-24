use anyhow::Context as _;
use anyhow::Result;
use directories::ProjectDirs;
use include_dir::include_dir;
use include_dir::{Dir, DirEntry};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
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
pub(crate) fn sandbox_exec(
    rendered: &str,
    path: &Path,
    args: &[String],
    home: &Option<PathBuf>,
) -> Result<()> {
    let envs = std::env::vars().map(|(k, v)| {
        if k == "HOME" {
            if let Some(home) = home {
                return (k, home.to_string_lossy().into_owned());
            }
        }
        (k, v)
    });
    Command::new("sandbox-exec")
        .envs(envs)
        .arg("-p")
        .arg(rendered)
        .arg(path)
        .args(args)
        .spawn()?
        .wait()?;
    Ok(())
}

/// Check if the library directory contains a given name
pub(crate) fn library_path_contains(name: &str) -> Result<bool> {
    let p = get_library_path()?;

    let d = WalkDir::new(p)
        .into_iter()
        .filter_map(|d| d.ok())
        .find(|d| d.path().to_str().unwrap_or("").contains(name))
        .is_some();
    Ok(d)
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
