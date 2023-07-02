#[cfg(debug_assertions)]
use axum::extract::ConnectInfo;
use axum::extract::{FromRequest, MatchedPath};
use axum::http::Request;
use axum::response::Html;
use axum::RequestPartsExt;
use axum::async_trait;
use minijinja::value::Value;

use std::collections::HashMap;
#[cfg(debug_assertions)]
use std::net::SocketAddr;

#[cfg(debug_assertions)]
use crate::TEMPLATES;

pub struct Template(pub String, pub HashMap<String, String>);

impl Template {
    pub fn render(self, context: Value) -> Html<String> {
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
        context.insert("form".into(), self.1.into());

        crate::render::render(self.0, context.into())
    }
}

#[async_trait]
impl<S, B> FromRequest<S, B> for Template
where
    axum::Form<HashMap<String, String>>: FromRequest<S, B>,
    S: Send + Sync,
    B: Send + 'static,
{
    type Rejection = ();

    #[cfg(not(debug_assertions))]
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
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

        let endpoint = parts
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
            Ok(axum::Form(mut form)) => {
                // 1: If this IP hasn't recorded any forms *at all*, create an empty history table.
                if !TEMPLATES.forms.read().unwrap().contains_key(&ip) {
                    TEMPLATES
                        .forms
                        .write()
                        .unwrap()
                        .insert(ip, Default::default());
                }

                // important: block-scoped to avoid holding locks during an .await
                {
                    let endpoint_form_history = TEMPLATES.forms.read().unwrap();
                    let ip_form_instances = endpoint_form_history.get(&ip).unwrap();

                    // 2: If this IP has recorded a form for *this endpoint*, create an empty history for this endpoint.
                    if !ip_form_instances.read().unwrap().contains_key(&endpoint) {
                        ip_form_instances
                            .write()
                            .unwrap()
                            .insert(endpoint.to_string(), Default::default());
                    }

                    // 3: The clientside HMR will assign an index for each element created.
                    //     when we request a reload, the client will give us the indexes for each element
                    //     that's still loaded. We can get something like [1, 3, 4] (elements 0 and 2 have been deleted),
                    //     and since the client updates each element in order, we can pop off the front of the indexes.
                    let top_form_index = ip_form_instances
                        .write()
                        .unwrap()
                        .get_mut(&endpoint)
                        .unwrap()
                        .indexes
                        .pop_front();

                    // 4: If there's an existing index, that means we're re-rendering an existing element with HMR magic.
                    //     Hypermedia relies on form data, so we'll reuse the existing form data so we don't have to re-submit it.
                    if let Some(existing_index) = top_form_index {
                        form = ip_form_instances
                            .read()
                            .unwrap()
                            .get(&endpoint)
                            .unwrap()
                            .contents[existing_index as usize]
                            .clone();
                    }
                    // 5: If this element is new, we'll save the requested form data since we can't be in HMR at this point.
                    else {
                        ip_form_instances
                            .write()
                            .unwrap()
                            .get_mut(&endpoint)
                            .unwrap()
                            .contents
                            .push(form.clone());
                    }
                }

                Ok(Self(endpoint, form))
            }
            Err(_) => Err(()),
        }
    }
}
