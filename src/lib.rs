#![forbid(unsafe_code)]

#[macro_use]
mod log;

#[cfg(debug_assertions)]
mod hmr;

pub mod config;
mod framework;
mod render;
mod runtime;
pub mod style;
mod template;
pub use framework::prelude;

#[cfg(debug_assertions)]
use std::collections::VecDeque;

#[cfg(debug_assertions)]
use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

pub use minijinja::context as _ctx;
use once_cell::sync::{Lazy, OnceCell};
use std::net::TcpListener;
pub use template::*;

pub mod reexports {
    pub use lightningcss;
    pub use minijinja;
}

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

pub(crate) fn template_dir() -> PathBuf {
    TEMPLATE_DIR
        .get()
        .cloned()
        .unwrap_or(PathBuf::from("templates"))
}

pub fn bind(addr: &'static str) -> TcpListener {
    let listener =
        TcpListener::bind(addr).unwrap_or_else(|_| panic!("Failed to bind to address: {addr}"));

    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => {
            error!("Network error while retrieving port: \"{e}\". Defaulting to port 80");
            80
        }
    };

    if let Ok(addrs) = if_addrs::get_if_addrs() {
        info!("Listening on:");
        addrs
            .into_iter()
            .filter(|i| !i.name.starts_with("br-") && !i.name.starts_with("docker"))
            .map(|i| i.ip())
            .filter(IpAddr::is_ipv4)
            .for_each(|i| {
                data!("http://{}:{}", i, port);
            })
    }

    listener
}

#[cfg(debug_assertions)]
type DB<T, U> = Mutex<HashMap<T, Mutex<U>>>;

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
pub(crate) struct TemplateSourceData {
    pub source: String,
    pub can_skip_rendering: bool,
}

#[cfg(not(debug_assertions))]
pub(crate) static TEMPLATES: Lazy<HashMap<String, TemplateSourceData>> = Lazy::new(|| {
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
                        .trim_end_matches(template_extension())
                        .replace("index", "")
                ),
                TemplateSourceData {
                    source: t.clone(),
                    can_skip_rendering: !(t.contains("{{")
                        || t.contains("}}")
                        || t.contains("{%")
                        || t.contains("%}")),
                },
            );
            acc
        })
});

#[cfg(debug_assertions)]
pub(crate) fn endpointof(path: &str) -> Option<&str> {
    let without_extension = path.trim_end_matches(template_extension());
    if without_extension == "/index" {
        Some("/")
    } else if without_extension == "index" {
        Some("")
    } else {
        Some(without_extension)
    }
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
        if Path::new(endpoint).extension().is_none() {
            template_extension()
        } else {
            ""
        }
    )
}
