use crate::template::Template;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub(crate) enum TemplateError {
    Io(std::io::Error),
    Parse(ron::de::SpannedError),
}

impl From<std::io::Error> for TemplateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<ron::de::SpannedError> for TemplateError {
    fn from(e: ron::de::SpannedError) -> Self {
        Self::Parse(e)
    }
}
impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}
impl std::error::Error for TemplateError {}

const TEMPLATE_EXT: &str = "ron";

pub fn template_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|p| p.join("clckwrd").join("templates"))
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

pub(crate) fn load_from_file(p: &Path) -> Result<Template, TemplateError> {
    let contents = fs::read_to_string(p)?;
    Ok(ron::from_str(&contents)?)
}

pub(crate) fn load_template(path: Option<&Path>) -> Template {
    let Some(path) = path else {
        return Template::default();
    };
    load_from_file(path).unwrap_or_else(|err| {
        eprintln!("ERROR loading template {}: {err:?}", path.display());
        Template::default()
    })
}
