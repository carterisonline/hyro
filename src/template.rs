use axum::async_trait;
#[cfg(debug_assertions)]
use axum::extract::ConnectInfo;
use axum::extract::{FromRequest, MatchedPath};
use axum::http::Request;
use axum::response::Html;
use axum::RequestPartsExt;
use minijinja::value::Value;
use once_cell::sync::OnceCell;

use std::borrow::Cow;
use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::net::SocketAddr;

use crate::context;
#[cfg(debug_assertions)]
use crate::TEMPLATES;

pub(crate) static TEMPLATE_EXTENSION: OnceCell<String> = OnceCell::new();

pub(crate) fn template_extension() -> &'static str {
    match TEMPLATE_EXTENSION.get() {
        Some(s) => s,
        None => ".html.jinja2",
    }
}

pub struct Template(pub String, pub HashMap<String, String>);

impl Template {
    pub fn render(self, context: Value) -> Html<Cow<'static, str>> {
        if context.is_undefined() {
            if self.1.is_empty() {
                crate::render::render(self.0, context)
            } else {
                crate::render::render(self.0, context!(form => self.1))
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
                context.insert("form".into(), self.1.into());
            }

            crate::render::render(self.0, context.into())
        }
    }
}

#[async_trait]
impl<S, B> FromRequest<S, B> for Template
where
    axum::Form<HashMap<String, String>>: FromRequest<S, B>,
    S: Send + Sync + std::fmt::Debug,
    B: Send + 'static + std::fmt::Debug,
{
    type Rejection = ();

    #[cfg(not(debug_assertions))]
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let endpoint = parts
            .extract::<MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .unwrap();

        let req = Request::from_parts(parts, body);

        match axum::Form::<HashMap<String, String>>::from_request(req, state).await {
            Ok(axum::Form(form)) => Ok(Self(endpoint, form)),
            Err(_) => Err(()),
        }
    }

    #[cfg(debug_assertions)]
    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let this_endpoint = parts
            .extract::<MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .unwrap();

        let ip = parts
            .extract::<ConnectInfo<SocketAddr>>()
            .await
            .unwrap()
            .ip();

        let req = Request::from_parts(parts, body);

        match axum::Form::from_request(req, state).await {
            Ok(axum::Form(form_from_request)) => {
                let mut forms = TEMPLATES.forms.lock();
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
                        return Ok(Self(
                            this_endpoint,
                            form_history.contents[oldest_outdated_element_id as usize].clone(),
                        ));
                    }

                    None => {
                        // 5: If there's nothing else in the queue, then we're rendering a new element!
                        //     We'll save the requested form data instead and return the requested form.
                        form_history.contents.push(form_from_request.clone());

                        return Ok(Self(this_endpoint, form_from_request));
                    }
                }
            }
            Err(_) => Err(()),
        }
    }
}
