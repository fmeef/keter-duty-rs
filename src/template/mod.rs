use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{utils, Args};
use anyhow::Result;
use once_cell::sync::OnceCell;
use tera::{Context, Function, Tera, Value};
use whoami::username;

// rustc is very weird about HRTBs, use hack to remove HRTBs for our closure
// to bind "self" to tera::Function. for a specific lifetime
// This only works because we use the custom self type Arc<Self>
fn bind<F>(func: F) -> impl Function
where
    F: Fn(&HashMap<String, Value>) -> Result<Value, tera::Error> + Send + Sync,
{
    func
}

/// sandbox-exec is very particular about trailing slashes on paths.
fn remove_trailing_slash(path: &[PathBuf]) -> Vec<String> {
    path.iter()
        .map(|v| remove_trailing_slash_single(v))
        .collect()
}

/// sandbox-exec is very particular about trailing slashes on paths.
fn remove_trailing_slash_single(v: &Path) -> String {
    let v = v.to_string_lossy();
    if v.ends_with("/") {
        v[0..v.len() - 1].to_owned()
    } else {
        v.into_owned()
    }
}

pub(crate) struct SandboxTemplate {
    args: Arc<Args>,
    tera: OnceCell<Tera>,
}
impl SandboxTemplate {
    pub(crate) fn new(args: Arc<Args>) -> Arc<Self> {
        let s = Self {
            args,
            tera: OnceCell::new(),
        };
        Arc::new(s)
    }

    pub(crate) fn get_tera(self: &Arc<Self>) -> Result<()> {
        let library_glob = utils::get_library_path()
            .expect("failed to get library path")
            .join("**.sb")
            .to_str()
            .expect("invalid UTF-8 in library path")
            .to_owned();

        let mut tera = Tera::new(&library_glob).expect("failed to initialize Tera");
        let me = self.clone();
        let f = bind(move |path| me.include(path));
        tera.register_function("include", f);
        self.tera.set(tera).unwrap();
        Ok(())
    }

    pub(crate) fn get_dirs_list(&self, dirs: &[PathBuf]) -> Result<(Vec<String>, Vec<String>)> {
        let mut res = remove_trailing_slash(dirs);
        let mut ancestors = dirs
            .iter()
            .flat_map(|v| v.ancestors())
            .map(|v| remove_trailing_slash_single(v))
            .collect::<Vec<String>>();

        if self.args.cwd {
            let dir = std::env::current_dir()?;
            let cwd = remove_trailing_slash_single(&dir);
            let da = dir.ancestors().map(|v| remove_trailing_slash_single(v));
            res.push(cwd);
            ancestors.extend(da);
        }
        Ok((res, ancestors))
    }

    /// Render a single template
    pub(crate) fn run_template<T: AsRef<str>>(&self, template: T) -> Result<String> {
        let mut context = Context::new();
        context.insert("username", &username());

        let (dirs, mut ancestors) = self.get_dirs_list(&self.args.dir)?;

        let (rodirs, roancestors) = self.get_dirs_list(&self.args.ro)?;

        ancestors.extend_from_slice(&roancestors);

        context.insert("rwfiles", &dirs);
        context.insert("ancestors", &ancestors);
        context.insert("rofiles", &rodirs);
        context.insert("executable", &self.args.exe);
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
