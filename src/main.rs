use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use template::{ARGS, TERA};
use utils::library_path_contains;
pub(crate) mod template;
pub(crate) mod utils;

#[derive(Parser, Debug)]
/// Wrapper for macos sandbox-exec (seatbelt) scripts
#[clap(author, version, about)]
struct Args {
    /// Relative path of the sandbox config
    name: String,

    /// Path of the executable to sandbox
    exe: PathBuf,

    /// Alternative templates directory
    templates_dir: Option<PathBuf>,

    /// allow access to the current working directory
    #[clap(takes_value = false, short, long)]
    cwd: bool,

    /// List of directories to allow
    #[clap(short, long)]
    dir: Vec<PathBuf>,

    /// Arguments to pass to the executable
    #[clap(last = true)]
    args: Vec<String>,
}

fn run_sb() -> Result<()> {
    let library_path = utils::get_library_path()?;
    utils::populate_sb_tree(&utils::SANDBOX_DIR, &library_path)?;

    let profile = TERA.run_template(&ARGS.name)?;
    utils::sandbox_exec(&profile, &ARGS.exe, &ARGS.args)?;

    Ok(())
}

fn main() -> Result<()> {
    TERA.get_tera().unwrap();
    if utils::SANDBOX_DIR.contains(&ARGS.name) || library_path_contains(&ARGS.name).unwrap() {
        crate::run_sb().unwrap();
    } else {
        println!("config {} doesn't exist", ARGS.name);
    }
    Ok(())
}
