use std::{collections::HashMap, path::PathBuf};

use crate::{utils, Args};
use anyhow::Result;
use clap::Parser;
use once_cell::sync::{Lazy, OnceCell};
use tera::{Context, Function, Tera, Value};
use whoami::username;

pub(crate) static ARGS: Lazy<Args> = Lazy::new(|| Args::parse());

pub(crate) static TERA: Lazy<SandboxTemplate> = Lazy::new(|| SandboxTemplate::new(&ARGS));

fn bind<F>(func: F) -> impl Function
where
    F: Fn(&HashMap<String, Value>) -> Result<Value, tera::Error> + Send + Sync,
{
    func
}

pub(crate) struct SandboxTemplate<'a> {
    args: &'a Args,
    tera: OnceCell<Tera>,
}
impl SandboxTemplate<'static> {
    pub(crate) fn new(args: &'static Args) -> Self {
        Self {
            args,
            tera: OnceCell::new(),
        }
    }

    pub(crate) fn get_tera(&'static self) -> Result<()> {
        let library_glob = utils::get_library_path()
            .expect("failed to get library path")
            .join("**.sb")
            .to_str()
            .expect("invalid UTF-8 in library path")
            .to_owned();

        let mut tera = Tera::new(&library_glob).expect("failed to initialize Tera");
        let f = bind(|path| self.include(path));
        tera.register_function("include", f);
        self.tera.set(tera).unwrap();
        Ok(())
    }

    pub(crate) fn get_dirs_list(&self) -> Result<Vec<PathBuf>> {
        let mut res = self.args.dir.clone();
        if self.args.cwd {
            let cwd = std::env::current_dir()?;
            res.push(cwd);
        }
        Ok(res)
    }

    /// Render a single template
    pub(crate) fn run_template<T: AsRef<str>>(&self, template: T) -> Result<String> {
        let mut context = Context::new();
        context.insert("username", &username());

        self.get_dirs_list()?;
        let template = self
            .tera
            .get()
            .unwrap()
            .render(template.as_ref(), &context)?;
        Ok(template)
    }

    /// Handler for tera "include" function. Renders and inserts an additional template
    fn include(&self, path: &HashMap<String, Value>) -> Result<Value, tera::Error> {
        let template = path
            .get("path")
            .ok_or("invalid name")?
            .as_str()
            .ok_or("failed")?;

        self.run_template(template)
            .map(Value::String)
            .map_err(|e| tera::Error::msg(e))
    }
}
