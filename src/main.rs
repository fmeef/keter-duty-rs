use anyhow::{Context as _, Result};
use clap::Parser;
use directories::ProjectDirs;
use include_dir::{include_dir, Dir, DirEntry};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tera::{Context, Tera, Value};
use whoami::username;

/// Default sandbox templates
static SANDBOX_DIR: Dir<'_> = include_dir!("sb");

static TERA: Lazy<Tera> = Lazy::new(|| {
    let library_glob = get_library_path()
        .expect("failed to get library path")
        .join("**.sb")
        .to_str()
        .expect("invalid UTF-8 in library path")
        .to_owned();

    let mut tera = Tera::new(&library_glob).expect("failed to initialize Tera");
    tera.register_function("include", include);
    tera
});

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

/// Copy over default templates to config directory.
/// Skips existing files, unlike [Dir::extract].
fn populate_sb_tree<T: AsRef<Path>>(dir: &Dir<'_>, base_path: &T) -> Result<()> {
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

/// Handler for tera "include" function. Renders and inserts an additional template
fn include(path: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let template = path
        .get("path")
        .ok_or("invalid name")?
        .as_str()
        .ok_or("failed")?;

    run_template(&TERA, template).map(Value::String)
}

/// Render a single template
fn run_template<T: AsRef<str>>(tera: &Tera, template: T) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("username", &username());
    tera.render(template.as_ref(), &context)
}

/// Gets the library path including the user's home directory
fn get_library_path() -> Result<PathBuf> {
    let project_dirs =
        ProjectDirs::from("", "", "KeterDuty").context("Failed to determine project paths")?;

    let data_dir = project_dirs.data_dir();
    fs::create_dir_all(data_dir).context("Failed to ensure data dir exists")?;
    Ok(data_dir.to_owned())
}

/// Runs sandbox-exec process with rendered template
fn sandbox_exec(rendered: &str, path: &Path, args: &[String]) -> Result<()> {
    Command::new("sandbox-exec")
        .arg("-p")
        .arg(rendered)
        .arg(path)
        .args(args)
        .output()?;
    Ok(())
}

fn run_sb(args: Args) -> Result<()> {
    let library_path = get_library_path()?;
    populate_sb_tree(&SANDBOX_DIR, &library_path)?;

    let profile = run_template(&TERA, &args.name)?;
    sandbox_exec(&profile, &args.exe, &args.args)?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if SANDBOX_DIR.contains(&args.name) {
        run_sb(args)?;
    } else {
        println!("config {} doesn't exist", args.name);
    }

    Ok(())
}
