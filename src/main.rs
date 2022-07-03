use anyhow::{anyhow, Result};
use clap::Parser;
use include_dir::{include_dir, Dir, DirEntry};
use lazy_static::{__Deref, lazy_static};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{create_dir_all, write},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
};
use tera::{Context, Tera, Value};
use whoami::username;

/// Default sandbox templates
static SANDBOX_DIR: Dir<'_> = include_dir!("sb");

/// Directory to store default and user templates
static LIBRARY_DIR: &str = "Library/Application Support/KeterDuty";

lazy_static! {
    static ref TERA: Tera = {
        let library_path = get_library_path().expect("failed to get library path");
        let mut tera = Tera::new(&format!("{}/**/*.sb", library_path.to_string_lossy()))
            .expect("failed to setup tera");
        tera.register_function("include", &include);
        tera
    };
}

#[derive(Parser, Debug)]
/// Wrapper for macos sandbox-exec (seatbelt) scripts
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Relative path of the sandbox config
    #[clap(value_parser)]
    name: PathBuf,

    /// Path of the executable to sandbox
    #[clap(value_parser)]
    exe: PathBuf,

    /// Alternative templates directory
    #[clap(value_parser)]
    templates_dir: Option<PathBuf>,

    /// Arguments to pass to the executable
    #[clap(value_parser)]
    args: Vec<String>,
}

/// Get the path of the user's home directory
fn get_home() -> Result<PathBuf> {
    let mut buf = PathBuf::from_str("/Users/").unwrap();
    buf.push(PathBuf::from_str(&username()).unwrap());
    if buf.exists() {
        Ok(buf)
    } else {
        Err(anyhow!("Path does not exist"))
    }
}

/// Copy over default templates to config directory, skipping any existing templates
fn populate_sb_tree<'a, T: AsRef<Path>>(dir: &Dir<'a>, base_path: &T) -> Result<()> {
    let base_path = base_path.as_ref();
    for entry in dir.entries() {
        let path = base_path.join(entry.path());
        match entry {
            DirEntry::Dir(d) => {
                create_dir_all(&path)?;
                populate_sb_tree(d, &base_path)?;
            }
            DirEntry::File(file) => {
                if !path.exists() {
                    write(path, file.contents())?;
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
    let mut library_path = get_home()?;
    library_path.push(PathBuf::from_str(LIBRARY_DIR)?);
    Ok(library_path)
}

/// Runs sandbox-exec process with rendered template
fn sandbox_exec<I, S>(rendered: &str, path: &PathBuf, args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
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
    SANDBOX_DIR.extract(&library_path)?;
    populate_sb_tree(&SANDBOX_DIR, &library_path)?;
    let template = run_template(TERA.deref(), &args.name.to_string_lossy())?;
    sandbox_exec(&template, &args.exe, args.args.into_iter())?;
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    if SANDBOX_DIR.contains(&args.name) {
        run_sb(args)?;
    } else {
        println!(
            "config {} doesn't exist",
            args.name.to_str().expect("path contains invalid unicode")
        );
    }

    Ok(())
}
