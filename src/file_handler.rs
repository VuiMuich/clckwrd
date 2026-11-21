use crate::template::Template;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const TEMPLATE_EXT: &str = "ron";

pub fn template_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|p| p.join("uhrwerk").join("templates"))
}

pub(crate) fn find_template(name: &str) -> Option<PathBuf> {
    if name.contains(['/', '\\']) || name.starts_with('.') {
        return None;
    }
    let path = template_dir()?.join(format!("{name}.{TEMPLATE_EXT}"));
    path.is_file().then_some(path)
}

pub(crate) fn list_templates() -> Vec<String> {
    let Some(dir) = template_dir() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == TEMPLATE_EXT))
        .filter_map(|p| p.file_stem()?.to_str().map(str::to_owned))
        .collect();
    names.sort();
    names
}

pub(crate) fn load_template(template_path: Option<String>) -> Template {
    load_from_file(&template_path.unwrap())
        .map_err(|err| println!("ERROR loading template: {:?}", err))
        .unwrap_or_default()
}

pub(crate) fn load_from_file(p: &str) -> Result<Template, Option<ron::de::SpannedError>> {
    if Path::new(&p).exists() {
        let contents = fs::read_to_string(p);
        let template: Result<Template, ron::de::SpannedError> =
            ron::from_str(contents.unwrap().as_str());
        return match template {
            Ok(t) => Ok(t),
            Err(e) => Err(Some(e)),
        };
    }
    // This is a ugly hack until proper Errors get implemented
    Err(None)
}
