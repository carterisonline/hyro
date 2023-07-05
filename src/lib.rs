#[cfg(debug_assertions)]
mod hmr;

mod render;
mod router;
pub mod style;
mod template;

#[cfg(debug_assertions)]
use std::collections::VecDeque;

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
#[cfg(debug_assertions)]
use std::sync::RwLock;

pub use minijinja::context as _ctx;
use once_cell::sync::{Lazy, OnceCell};
pub use router::*;
use std::net::TcpListener;
pub use template::*;

#[doc(hidden)]
pub fn _empty_context() -> minijinja::value::Value {
    minijinja::value::Value::UNDEFINED
}

#[macro_export]
macro_rules! context {
    () => {
        $crate::_empty_context()
    };

    ($($key:ident $(=> $value:expr)?),* $(,)?) => {
        $crate::_ctx!($($key $(=> $value)?),*)
    };
}

static TEMPLATE_DIR: OnceCell<PathBuf> = OnceCell::new();

pub fn set_template_dir<T: AsRef<Path>>(dir: T) -> Result<(), PathBuf> {
    TEMPLATE_DIR.set(dir.as_ref().to_path_buf())
}

pub(crate) fn template_dir() -> PathBuf {
    TEMPLATE_DIR
        .get()
        .cloned()
        .unwrap_or(PathBuf::from("templates"))
}

pub async fn bind(addr: &'static str) -> TcpListener {
    let listener =
        TcpListener::bind(addr).unwrap_or_else(|_| panic!("Failed to bind to address: {addr}"));

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            eprintln!("Network error while retrieving port: \"{e}\". Defaulting to port 80");
            80
        }
    };

    if let Ok(addrs) = if_addrs::get_if_addrs() {
        println!("Listening on:");
        addrs
            .into_iter()
            .filter(|i| !i.name.starts_with("br-") && !i.name.starts_with("docker"))
            .map(|i| i.ip())
            .filter(IpAddr::is_ipv4)
            .for_each(|i| {
                println!("http://{}:{}", i, port);
            })
    }

    listener
}

#[cfg(debug_assertions)]
type DB<T, U> = RwLock<HashMap<T, RwLock<U>>>;

#[cfg(debug_assertions)]
#[derive(Debug, Default, Clone)]
pub(crate) struct TemplateFormData {
    pub contents: Vec<HashMap<String, String>>,
    pub indexes: VecDeque<u32>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub(crate) struct Templates {
    pub sources: DB<String, String>,
    pub forms: DB<IpAddr, HashMap<String, TemplateFormData>>,
}

#[cfg(debug_assertions)]
pub(crate) static TEMPLATES: Lazy<Templates> = Lazy::new(Templates::default);

#[cfg(not(debug_assertions))]
pub(crate) static TEMPLATES: Lazy<HashMap<String, (String, bool)>> = Lazy::new(|| {
    walkdir::WalkDir::new(template_dir())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|i| i.path().is_file())
        .fold(HashMap::new(), |mut acc, entry| {
            let dir = entry.path().to_string_lossy().to_string();
            let t = std::fs::read_to_string(dir).unwrap();
            acc.insert(
                format!(
                    "/{}",
                    entry
                        .path()
                        .strip_prefix(template_dir())
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                        .trim_end_matches(".html.jinja2")
                        .replace("index", "")
                ),
                (
                    t.clone(),
                    t.contains("{{") || t.contains("}}") || t.contains("{%") || t.contains("%}"),
                ),
            );
            acc
        })
});

#[cfg(debug_assertions)]
pub(crate) fn endpointof(path: &str) -> Option<String> {
    Some(path.trim_end_matches(".html.jinja2").to_string())
}

pub(crate) fn path_of_endpoint<S: AsRef<str>>(endpoint: S) -> String {
    let endpoint = endpoint.as_ref();
    format!(
        "{}{}",
        if endpoint.ends_with('/') {
            format!("{endpoint}index")
        } else {
            endpoint.into()
        }
        .trim_start_matches('/'),
        if PathBuf::from(endpoint).extension().is_none() {
            ".html.jinja2"
        } else {
            ""
        }
    )
}
