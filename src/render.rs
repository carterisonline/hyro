use minijinja::value::{Value, ValueKind};
use parking_lot::Mutex;
use std::borrow::Cow;

use minijinja::Environment;
use once_cell::sync::Lazy;
use tap::Tap;

use crate::framework::*;
use crate::{path_of_endpoint, template_extension, TEMPLATES};

const HMR_ENABLED: bool = cfg!(debug_assertions);

pub(crate) static ENVIRONMENT: Lazy<Mutex<Environment>> = Lazy::new(|| {
    Mutex::new(Environment::new().tap_mut(|env| {
        env.add_global("hmr", HMR_ENABLED);
        env.add_function("module", module);
    }))
});

fn module(path: String, form: Option<Value>) -> Result<String, minijinja::Error> {
    let path = path_of_endpoint(path);
    let path = path.trim_end_matches(template_extension());

    match (form.as_ref().map(Value::kind), form) {
        (Some(ValueKind::Map), Some(form)) => match serde_urlencoded::to_string(form) {
            Ok(form) => Ok(format!(
                r#"<div hx-trigger="revealed" hx-swap="outerHTML" hx-get="{path}?{form}"></div>"#,
            )),
            Err(e) => Err(minijinja::Error::new(
                minijinja::ErrorKind::BadSerialization,
                format!("invalid form data: {e}"),
            )),
        },
        (Some(_), _) => Err(minijinja::Error::new(
            minijinja::ErrorKind::BadSerialization,
            "form data should be a map",
        )),
        (None, _) => Ok(format!(
            r#"<div hx-trigger="revealed" hx-swap="outerHTML" hx-get="{path}"></div>"#
        )),
    }
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
    template_name: S,
    value: minijinja::value::Value,
) -> RenderedTemplate {
    let template = TEMPLATES.get(template_name.as_ref()).unwrap();

    if template.can_skip_rendering {
        return into_rendered_template(Cow::Borrowed(template.source.as_str()));
    } else {
        match ENVIRONMENT.lock().render_str(&template.source, value) {
            Ok(t) => return into_rendered_template(Cow::Owned(t)),
            Err(e) => {
                error!("Error while rendering {}: {:?}", template_name.as_ref(), e);
                return into_rendered_template(Cow::Borrowed(""));
            }
        }
    }
}

#[cfg(debug_assertions)]
pub(crate) fn render<S: AsRef<str> + std::fmt::Debug>(
    template: S,
    value: minijinja::value::Value,
) -> RenderedTemplate {
    init_template(template.as_ref());

    let template_sources = TEMPLATES.sources.lock();
    let template_source = template_sources.get(template.as_ref()).unwrap().lock();

    let maybe_rendered = ENVIRONMENT.lock().render_str(
        &inject_hmr(&inject_template_path(template.as_ref(), &template_source)),
        value,
    );

    match maybe_rendered {
        Ok(t) => into_rendered_template(Cow::Owned(t)),
        Err(e) => {
            error!("Error while rendering {}: {}", template.as_ref(), e);
            into_rendered_template(Cow::Borrowed(""))
        }
    }
}

#[cfg(debug_assertions)]
fn init_template(template_name: &str) {
    let template_exists = TEMPLATES.sources.lock().contains_key(template_name);
    if !template_exists {
        reload_template(
            &crate::template_dir()
                .join(std::path::Path::new(template_name.trim_start_matches('/')))
                .display()
                .to_string(),
        );
    }
}

#[cfg(debug_assertions)]
pub(crate) fn reload_template(template_name: &str) {
    let _path = &path_of_endpoint(template_name);
    let path = std::path::Path::new(_path);

    let template_source = std::fs::read_to_string(path).unwrap();

    match minijinja::machinery::parse(
        &template_source,
        &std::env::current_dir()
            .unwrap()
            .join(path)
            .display()
            .to_string(),
    ) {
        Ok(_) => {
            TEMPLATES.sources.lock().insert(
                crate::endpointof(_path.trim_start_matches("templates"))
                    .unwrap()
                    .into(),
                Mutex::new(template_source),
            );
        }
        Err(e) => {
            error!("{e}");
        }
    }
}
