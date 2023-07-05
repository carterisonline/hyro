use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use minijinja::value::Value;
use once_cell::sync::Lazy;

static PRERENDERED: Lazy<HashMap<PathBuf, String>> = Lazy::new(HashMap::new);
static CACHED_RENDER_ASSOC: Lazy<HashMap<PathBuf, usize>> = Lazy::new(HashMap::new);

type Renderer = dyn Fn(PathBuf, Option<Value>) -> String;

#[derive(Default)]
pub struct App {
    handlers: Vec<(FileType, Box<Renderer>)>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_handler(mut self, file_type: FileType, renderer: Box<Renderer>) -> Self {
        self.handlers.push((file_type, renderer));
        self
    }

    pub fn render(&self, path: PathBuf) -> std::io::Result<Cow<'static, str>> {
        if let Some(prerendered_content) = PRERENDERED.get(&path) {
            Cow::Borrowed(&prerendered_content)
        } else if let Some(render_assoc) = CACHED_RENDER_ASSOC
            .get(&path)
            .map(|i| Some(*i))
            .unwrap_or_else(|| {
                for (i, (handler_type, _)) in self.handlers.iter().enumerate() {
                    if handler_type.matches_file(path) {
                        return Some(i);
                    }
                }
                return None;
            }) {

            
        
        } else {
            let content = std::fs::read_to_string(path)?;
            PRERENDERED.insert(path.clone(), content.clone());

            return Ok(Cow::Borrowed(PRERENDERED.get(&path).unwrap()));
        }
    }
}

#[derive(PartialEq, Eq, Hash)]
pub enum FileType {
    Extension(String),
    Mime(String),
}

impl FileType {
    pub fn matches_file<P: AsRef<Path>>(&self, path: P) -> bool {
        match self {
            FileType::Extension(ext) => {
                path.as_ref().extension().map(OsStr::to_str) == Some(Some(ext.as_str()))
            }
            FileType::Mime(mime) => match infer::get_from_path(path.as_ref()) {
                Ok(Some(t)) => t.mime_type() == mime.as_str(),
                _ => false,
            },
        }
    }
}
