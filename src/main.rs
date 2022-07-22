use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
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

    /// Arguments to pass to the executable
    args: Vec<String>,
}

fn run_sb(args: Args) -> Result<()> {
    let library_path = utils::get_library_path()?;
    utils::populate_sb_tree(&utils::SANDBOX_DIR, &library_path)?;

    let profile = template::run_template(&template::TERA, &args.name)?;
    utils::sandbox_exec(&profile, &args.exe, &args.args)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if utils::SANDBOX_DIR.contains(&args.name) || library_path_contains(&args.name)? {
        run_sb(args)?;
    } else {
        println!("config {} doesn't exist", args.name);
    }

    Ok(())
}
