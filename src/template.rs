use minijinja::value::Value;
use once_cell::sync::OnceCell;

use std::collections::HashMap;

use crate::context;
use crate::framework::*;

pub(crate) static TEMPLATE_EXTENSION: OnceCell<String> = OnceCell::new();

pub(crate) fn template_extension() -> &'static str {
    match TEMPLATE_EXTENSION.get() {
        Some(s) => s,
        None => ".html.jinja2",
    }
}

pub struct Template {
    pub path: String,
    pub form: HashMap<String, String>,
}

impl Template {
    pub fn render(self, context: Value) -> RenderedTemplate {
        if context.is_undefined() {
            if self.form.is_empty() {
                crate::render::render(self.path, context)
            } else {
                crate::render::render(self.path, context!(form => self.form))
            }
        } else {
            let mut context = context
                .try_iter()
                .unwrap()
                .fold(HashMap::new(), |mut h, k| {
                    h.insert(
                        k.as_str().unwrap().to_string(),
                        context.get_item(&k).unwrap(),
                    );
                    h
                });

            if !context.contains_key("form") {
                context.insert("form".into(), self.form.into());
            }

            crate::render::render(self.path, context.into())
        }
    }
}

#[cfg(debug_assertions)]
pub(crate) fn template_hydrate(
    ip: std::net::IpAddr,
    this_endpoint: String,
    form_from_request: HashMap<String, String>,
) -> Template {
    let mut forms = crate::TEMPLATES.forms.lock();
    // 1: If this IP hasn't recorded any forms *at all*, create an empty history table.
    forms.entry(ip).or_insert_with(Default::default);

    let mut ip_endpoint_history = forms.get(&ip).unwrap().lock();

    // 2: If this IP has not recorded a form for *this endpoint*, create an empty history for this endpoint.
    if !ip_endpoint_history.contains_key(&this_endpoint) {
        ip_endpoint_history.insert(this_endpoint.to_string(), Default::default());
    }

    let form_history = ip_endpoint_history.get_mut(&this_endpoint).unwrap();

    // 3: The clientside HMR will assign an index for each element created.
    //     when we request a reload, the client will give us the indexes for each element
    //     that's still loaded. We can get something like [1, 3, 4] (elements 0 and 2 have been deleted),
    //     and since the client updates each element in order, we can pop off the front of the indexes.
    match form_history.indexes.pop_front() {
        Some(oldest_outdated_element_id) => {
            // 4: If there's an existing index, that means we're re-rendering an existing element with HMR magic.
            //     Hypermedia relies on form data, so we'll reuse the existing form data so we don't have to re-submit it.
            Template {
                path: this_endpoint,
                form: form_history.contents[oldest_outdated_element_id as usize].clone(),
            }
        }

        None => {
            // 5: If there's nothing else in the queue, then we're rendering a new element!
            //     We'll save the requested form data instead and return the requested form.
            form_history.contents.push(form_from_request.clone());

            Template {
                path: this_endpoint,
                form: form_from_request,
            }
        }
    }
}
