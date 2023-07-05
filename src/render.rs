use std::borrow::Cow;
#[cfg(debug_assertions)]
use std::path::Path;
#[cfg(debug_assertions)]
use std::sync::RwLock;

use axum::response::Html;
use minijinja::Environment;
use once_cell::sync::Lazy;
#[cfg(debug_assertions)]
use tap::Pipe;
use tap::Tap;

#[cfg(debug_assertions)]
use crate::{endpointof, template_dir};
use crate::{path_of_endpoint, TEMPLATES};

#[cfg(debug_assertions)]
const HMR_ENABLED: bool = true;
#[cfg(not(debug_assertions))]
const HMR_ENABLED: bool = false;

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::new().tap_mut(|env| {
        env.add_global("hmr", HMR_ENABLED);
        env.add_function("module", module);
    })
});

fn module(path: String) -> Result<String, minijinja::Error> {
    let path = path_of_endpoint(path);
    let path = path.trim_end_matches(".html.jinja2");

    Ok(format!(
        r#"<div hx-trigger="revealed" hx-swap="outerHTML" hx-get="{path}"></div>"#
    ))
}

#[cfg(debug_assertions)]
fn inject_template_path(path: &str, template: &str) -> String {
    let loc = if let Some(stripped) = template.strip_prefix("<!DOCTYPE html>") {
        stripped.find('>').map(|i| i + 15)
    } else {
        template.find('>')
    };

    if let Some(insert_pos) = loc {
        format!(
            "{} hmr-path=\"{}\"{}",
            &template[..insert_pos],
            path,
            &template[insert_pos..]
        )
    } else {
        template.to_string()
    }
}

#[cfg(debug_assertions)]
fn inject_hmr(template: &str) -> String {
    if let Some(head_end_pos) = template.find("</head>") {
        format!(
            "{}\n\t<script>\n{}\n</script>\n{}",
            &template[..head_end_pos],
            include_str!("hmr.js"),
            &template[head_end_pos..]
        )
    } else {
        template.to_string()
    }
}

#[cfg(not(debug_assertions))]
pub(crate) fn render<S: AsRef<str> + std::fmt::Debug>(
    template: S,
    value: minijinja::value::Value,
) -> Html<Cow<'static, str>> {
    let t = TEMPLATES.get(template.as_ref()).unwrap();

    if t.1 {
        match ENVIRONMENT.render_str(&t.0, value) {
            Ok(t) => return Html(Cow::Owned(t)),
            Err(e) => {
                eprintln!("Error while rendering {}: {:?}", template.as_ref(), e);
                return Html(Cow::Borrowed(""));
            }
        }
    } else {
        return Html(Cow::Borrowed(t.0.as_str()));
    }
}

#[cfg(debug_assertions)]
pub(crate) fn render<S: AsRef<str> + std::fmt::Debug>(
    template: S,
    value: minijinja::value::Value,
) -> Html<Cow<'static, str>> {
    init_template(template.as_ref());

    TEMPLATES
        .sources
        .read()
        .unwrap()
        .get(template.as_ref())
        .unwrap()
        .read()
        .map_err(|_| {
            minijinja::Error::new(
                minijinja::ErrorKind::TemplateNotFound,
                "Internal retrieval error",
            )
        })
        .and_then(|t| {
            #[cfg(debug_assertions)]
            return Ok(Cow::Owned(ENVIRONMENT.render_str(
                &inject_hmr(&inject_template_path(template.as_ref(), &t)),
                value,
            )?));
        })
        .pipe(|t| match t {
            Ok(t) => Html(t),
            Err(e) => {
                eprintln!("Error while rendering {}: {:?}", template.as_ref(), e);
                Html(Cow::Borrowed(""))
            }
        })
}

#[cfg(debug_assertions)]
fn init_template(template: &str) {
    let is_none = TEMPLATES.sources.read().unwrap().get(template).is_none();
    if is_none {
        reload_template(template);
    }
}

#[cfg(debug_assertions)]
pub(crate) fn reload_template(template_name: &str) {
    let p = &path_of_endpoint(template_name);
    let path = Path::new(p);

    let template = std::fs::read_to_string(template_dir().join(path)).unwrap();
    TEMPLATES
        .sources
        .write()
        .unwrap()
        .insert(endpointof(template_name).unwrap(), RwLock::new(template));
}
