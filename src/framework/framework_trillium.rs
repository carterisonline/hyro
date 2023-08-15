use crate::template::Template;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

pub use trillium_websockets::WebSocketConn as WebSocket;

pub type RenderedTemplate = Cow<'static, str>;

pub fn into_rendered_template(cow: Cow<'static, str>) -> RenderedTemplate {
    cow
}

#[cfg(debug_assertions)]
pub async fn hmr_websocket(conn: WebSocket) {
    let ip = conn.peer_ip().unwrap();
    crate::hmr::hmr_handler(conn, ip).await;
}

#[cfg(debug_assertions)]
pub async fn websocket_recv(
    socket: &mut WebSocket,
) -> Option<Result<trillium_websockets::tungstenite::Message, trillium_websockets::Error>> {
    use futures_lite::StreamExt;

    socket.next().await
}

#[cfg(debug_assertions)]
pub async fn websocket_unwrap<T>(socket: async_channel::Send<'_, T>) {
    socket.await.unwrap();
}

#[cfg(debug_assertions)]
pub fn websocket_message_text(text: String) -> trillium_websockets::tungstenite::Message {
    trillium_websockets::tungstenite::Message::Text(text)
}

#[cfg(debug_assertions)]
pub fn websocket_message_binary(data: Vec<u8>) -> trillium_websockets::tungstenite::Message {
    trillium_websockets::tungstenite::Message::Binary(data)
}

#[cfg(debug_assertions)]
pub fn websocket_try_as_text(message: trillium_websockets::tungstenite::Message) -> Option<String> {
    match message {
        trillium_websockets::tungstenite::Message::Text(t) => Some(t),
        _ => None,
    }
}

#[cfg(debug_assertions)]
pub fn websocket_try_as_binary(
    message: trillium_websockets::tungstenite::Message,
) -> Option<Vec<u8>> {
    match message {
        trillium_websockets::tungstenite::Message::Binary(b) => Some(b),
        _ => None,
    }
}

pub trait RouterExt {
    fn with_hmr(self) -> Self;
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self;
}

impl RouterExt for trillium_router::Router {
    #[cfg(debug_assertions)]
    fn with_hmr(self) -> Self {
        crate::hmr::watch_templates();
        self.get("/hmr", trillium_websockets::websocket(hmr_websocket))
    }

    #[cfg(not(debug_assertions))]
    fn with_hmr(self) -> Self {
        self
    }

    #[cfg(debug_assertions)]
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self {
        crate::style::STYLE_MAIN_FILE
            .set(main_css_path.as_ref().to_path_buf())
            .unwrap();

        crate::hmr::watch_style();

        self.get(endpoint, main_css)
    }

    #[cfg(not(debug_assertions))]
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self {
        crate::style::MAIN_CSS
            .set(crate::style::transform_css(&main_css_path.as_ref().to_path_buf()).unwrap())
            .unwrap();

        self.get(endpoint, main_css)
    }
}

#[cfg(debug_assertions)]
async fn main_css(conn: trillium::Conn) -> trillium::Conn {
    conn.with_header("Content-Type", "text/css").with_body(
        crate::style::transform_css(crate::style::STYLE_MAIN_FILE.get().unwrap()).unwrap(),
    )
}

#[cfg(not(debug_assertions))]
async fn main_css(conn: trillium::Conn) -> trillium::Conn {
    conn.with_header("Content-Type", "text/css")
        .with_body(crate::style::MAIN_CSS.get().unwrap().as_str())
}

#[trillium::async_trait]
pub trait ConnExt {
    async fn template(&mut self) -> Template;
}

#[trillium::async_trait]
impl ConnExt for trillium::Conn {
    #[cfg(not(debug_assertions))]
    async fn template(&mut self) -> Template {
        let path = self.path().to_owned();
        let form: HashMap<String, String> =
            serde_urlencoded::from_str::<HashMap<String, String>>(self.querystring())
                .unwrap_or_default();

        Template { path, form }
    }

    #[cfg(debug_assertions)]
    async fn template(&mut self) -> Template {
        let path = self.path().to_owned();
        let ip = self.peer_ip().unwrap();

        let form: HashMap<String, String> =
            serde_urlencoded::from_str::<HashMap<String, String>>(self.querystring())
                .unwrap_or_default();

        crate::template::template_hydrate(ip, path, form)
    }
}
