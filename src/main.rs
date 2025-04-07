use anyhow::{anyhow, Result};
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

    /// Alternative templates directory
    #[clap(long, short)]
    templates_dir: Option<PathBuf>,

    /// allow access to the current working directory
    #[clap(takes_value = false, short, long)]
    cwd: bool,

    /// List of directories to allow
    #[clap(short, long)]
    dir: Vec<PathBuf>,

    /// List of readonly directories to allow
    #[clap(short, long)]
    ro: Vec<PathBuf>,

    /// Set $HOME env variable to alternate home
    #[clap(short, long)]
    home: Option<PathBuf>,

    /// Sets the TMPDIR environment variable (defaults to $HOME/tmp)
    #[clap(short, long)]
    tmp: Option<PathBuf>,

    /// Path of the executable to sandbox
    exe: PathBuf,

    /// Arguments to pass to the executable
    args: Vec<String>,
}

fn run_sb(mut args: Args) -> Result<()> {
    if let Some(ref home) = args.home {
        args.dir.push(home.clone());
    }

    let library_path = utils::get_library_path()?;
    utils::populate_sb_tree(&utils::SANDBOX_DIR, &library_path)?;

    let args = Arc::new(args);

    let templates = SandboxTemplate::new(Arc::clone(&args));
    templates.get_tera()?;

    let template = templates.run_template(&args.name)?;

    utils::sandbox_exec(&template, &args.exe, &args.args, &args.home, &args.tmp)?;

    Ok(())
}

fn fail_home() -> Result<()> {
    eprint!("Allowing write access to home directory. This is a potential security risk Are you sure? (y/n): ");
    let mut buf = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buf)?;
    if buf != "y" {
        return Err(anyhow!("aborted"));
    }
    Ok(())
}

// A common mistake is to enable the cwd flag in the home directory, causing everything to become writable
// attempt to prevent this
fn check_home(args: &Args) -> Result<()> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cry"))?;
    if args.cwd && std::env::current_dir()?.eq(&home) {
        fail_home()?;
    } else if let Some(home) = args.home.as_ref() {
        if dirs::home_dir().unwrap() == *home {
            fail_home()?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    check_home(&args)?;
    if utils::SANDBOX_DIR.contains(&args.name) || library_path_contains(&args.name).unwrap() {
        crate::run_sb(args).unwrap();
    } else {
        println!("config {} doesn't exist", args.name);
    }
    Ok(())
}
