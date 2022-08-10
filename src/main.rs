use anyhow::Result;
use clap::Parser;
use std::{path::PathBuf, sync::Arc};
use template::SandboxTemplate;
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

fn run_sb(args: Args) -> Result<()> {
    let library_path = utils::get_library_path()?;
    utils::populate_sb_tree(&utils::SANDBOX_DIR, &library_path)?;

    let args = Arc::new(args);

    let templates = SandboxTemplate::new(Arc::clone(&args));
    templates.get_tera()?;

    let template = templates.run_template(&args.name)?;

    utils::sandbox_exec(&template, &args.exe, &args.args)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if utils::SANDBOX_DIR.contains(&args.name) || library_path_contains(&args.name).unwrap() {
        crate::run_sb(args).unwrap();
    } else {
        println!("config {} doesn't exist", args.name);
    }
    Ok(())
}
