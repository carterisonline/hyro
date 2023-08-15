use std::borrow::Cow;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

pub use axum::extract::ws::WebSocket;
use axum::extract::FromRequest;
use axum::response::{Html, IntoResponse};
use axum::{async_trait, Router};

pub type RenderedTemplate = Html<Cow<'static, str>>;

pub fn into_rendered_template(cow: Cow<'static, str>) -> RenderedTemplate {
    Html(cow)
}

#[cfg(debug_assertions)]
pub async fn hmr_websocket(
    axum::extract::ConnectInfo(ip): axum::extract::ConnectInfo<SocketAddr>,
    ws: axum::extract::WebSocketUpgrade,
) -> axum::response::Response {
    let ip = ip.ip();
    ws.on_upgrade(move |socket| async move { crate::hmr::hmr_handler(socket, ip).await })
}

#[cfg(debug_assertions)]
pub async fn websocket_recv(
    socket: &mut WebSocket,
) -> Option<Result<axum::extract::ws::Message, axum::Error>> {
    socket.recv().await
}

#[cfg(debug_assertions)]
pub async fn websocket_unwrap<T: std::fmt::Debug>(
    socket: Result<usize, tokio::sync::broadcast::error::SendError<T>>,
) {
    socket.unwrap();
}

#[cfg(debug_assertions)]
pub fn websocket_message_text(text: String) -> axum::extract::ws::Message {
    axum::extract::ws::Message::Text(text)
}

#[cfg(debug_assertions)]
pub fn websocket_message_binary(data: Vec<u8>) -> axum::extract::ws::Message {
    axum::extract::ws::Message::Binary(data)
}

#[cfg(debug_assertions)]
pub fn websocket_try_as_text(message: axum::extract::ws::Message) -> Option<String> {
    match message {
        axum::extract::ws::Message::Text(t) => Some(t),
        _ => None,
    }
}

#[cfg(debug_assertions)]
pub fn websocket_try_as_binary(message: axum::extract::ws::Message) -> Option<Vec<u8>> {
    match message {
        axum::extract::ws::Message::Binary(b) => Some(b),
        _ => None,
    }
}

pub trait RouterExt<S, C> {
    /// Transforms the router, giving it permissions necessary for HMR during debug builds.
    /// Does nothing when building for release.
    #[cfg(debug_assertions)]
    fn into_service_with_hmr(
        self,
    ) -> axum::extract::connect_info::IntoMakeServiceWithConnectInfo<S, C>;
    #[cfg(not(debug_assertions))]
    fn into_service_with_hmr(self) -> axum::routing::IntoMakeService<Router>;
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self;
}

impl RouterExt<Router, SocketAddr> for axum::Router {
    #[cfg(debug_assertions)]
    fn into_service_with_hmr(
        self,
    ) -> axum::extract::connect_info::IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
        crate::hmr::watch_templates();
        self.route("/hmr", axum::routing::get(hmr_websocket))
            .layer(tower_http::trace::TraceLayer::new_for_http())
            .into_make_service_with_connect_info::<SocketAddr>()
    }
    #[cfg(not(debug_assertions))]
    fn into_service_with_hmr(self) -> axum::routing::IntoMakeService<Router> {
        self.into_make_service()
    }

    #[cfg(debug_assertions)]
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self {
        crate::style::STYLE_MAIN_FILE
            .set(main_css_path.as_ref().to_path_buf())
            .unwrap();

        crate::hmr::watch_style();

        self.route(endpoint, axum::routing::get(main_css))
    }

    #[cfg(not(debug_assertions))]
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self {
        crate::style::MAIN_CSS
            .set(crate::style::transform_css(&main_css_path.as_ref().to_path_buf()).unwrap())
            .unwrap();

        self.route(endpoint, axum::routing::get(main_css))
    }
}

#[cfg(debug_assertions)]
async fn main_css() -> impl IntoResponse {
    (
        [("Content-Type", "text/css")],
        crate::style::transform_css(crate::style::STYLE_MAIN_FILE.get().unwrap()).unwrap(),
    )
}

#[cfg(not(debug_assertions))]
async fn main_css() -> impl IntoResponse {
    (
        [("Content-Type", "text/css")],
        crate::style::MAIN_CSS.get().unwrap().as_str(),
    )
}

#[async_trait]
impl<S, B> FromRequest<S, B> for crate::template::Template
where
    axum::Form<HashMap<String, String>>: FromRequest<S, B>,
    S: Send + Sync + std::fmt::Debug,
    B: Send + 'static + std::fmt::Debug,
{
    type Rejection = ();

    #[cfg(not(debug_assertions))]
    async fn from_request(req: axum::http::Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;
        let (mut parts, body) = req.into_parts();

        let endpoint = parts
            .extract::<axum::extract::MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .unwrap();

        let req = axum::http::Request::from_parts(parts, body);

        match axum::Form::<HashMap<String, String>>::from_request(req, state).await {
            Ok(axum::Form(form)) => Ok(Self {
                path: endpoint,
                form,
            }),
            Err(_) => Err(()),
        }
    }

    #[cfg(debug_assertions)]
    async fn from_request(req: axum::http::Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        use axum::RequestPartsExt;

        let (mut parts, body) = req.into_parts();

        let this_endpoint = parts
            .extract::<axum::extract::MatchedPath>()
            .await
            .map(|path| path.as_str().to_owned())
            .unwrap();

        let ip = parts
            .extract::<axum::extract::ConnectInfo<SocketAddr>>()
            .await
            .unwrap()
            .ip();

        let req = axum::http::Request::from_parts(parts, body);

        axum::Form::from_request(req, state)
            .await
            .map(|form| crate::template::template_hydrate(ip, this_endpoint, form.0))
            .map_err(|_| ())
    }
}
