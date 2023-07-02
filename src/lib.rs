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
use std::path::PathBuf;
use std::sync::RwLock;

pub use minijinja::context;
use once_cell::sync::Lazy;
pub use router::*;
pub use template::*;
use tokio::net::TcpListener;

pub async fn bind(addr: &'static str) -> TcpListener {
    let listener = TcpListener::bind(addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to address: {addr}"));

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

type DB<T, U> = RwLock<HashMap<T, RwLock<U>>>;

#[cfg(debug_assertions)]
#[derive(Debug, Default, Clone)]
pub(crate) struct TemplateFormData {
    pub contents: Vec<HashMap<String, String>>,
    pub indexes: VecDeque<u32>,
}

#[derive(Debug, Default)]
pub(crate) struct Templates {
    pub sources: DB<String, String>,
    #[cfg(debug_assertions)]
    pub forms: DB<IpAddr, HashMap<String, TemplateFormData>>,
}

pub(crate) static TEMPLATES: Lazy<Templates> = Lazy::new(Templates::default);

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
