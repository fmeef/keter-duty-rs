use std::collections::HashMap;

use crate::utils;
use once_cell::sync::Lazy;
use tera::{Context, Tera, Value};
use whoami::username;

pub(crate) static TERA: Lazy<Tera> = Lazy::new(|| {
    let library_glob = utils::get_library_path()
        .expect("failed to get library path")
        .join("**.sb")
        .to_str()
        .expect("invalid UTF-8 in library path")
        .to_owned();

    let mut tera = Tera::new(&library_glob).expect("failed to initialize Tera");
    tera.register_function("include", include);
    tera
});

/// Render a single template
pub(crate) fn run_template<T: AsRef<str>>(tera: &Tera, template: T) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("username", &username());
    tera.render(template.as_ref(), &context)
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
