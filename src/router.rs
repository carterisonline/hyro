use std::net::SocketAddr;

#[cfg(debug_assertions)]
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::response::IntoResponse;
#[cfg(not(debug_assertions))]
use axum::routing::IntoMakeService;
use axum::Router;
use std::path::Path;
#[cfg(debug_assertions)]
use tower_http::trace::TraceLayer;

pub trait RouterExt<S, C> {
    /// Transforms the router, giving it permissions necessary for HMR during debug builds.
    /// Does nothing when building for release.
    #[cfg(debug_assertions)]
    fn into_service_with_hmr(self) -> IntoMakeServiceWithConnectInfo<S, C>;
    #[cfg(not(debug_assertions))]
    fn into_service_with_hmr(self) -> IntoMakeService<Router>;
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self;
}

impl RouterExt<Router, SocketAddr> for axum::Router {
    #[cfg(debug_assertions)]
    fn into_service_with_hmr(self) -> IntoMakeServiceWithConnectInfo<Router, SocketAddr> {
        crate::hmr::watch_templates();
        self.route(
            "/hmr",
            axum::routing::get(crate::hmr::hmr_websocket_handler),
        )
        .layer(TraceLayer::new_for_http())
        .into_make_service_with_connect_info::<SocketAddr>()
    }
    #[cfg(not(debug_assertions))]
    fn into_service_with_hmr(self) -> IntoMakeService<Router> {
        self.into_make_service()
    }

    #[cfg(debug_assertions)]
    fn with_bundled_css<P: AsRef<Path>>(self, endpoint: &str, main_css_path: P) -> Self {
        use crate::hmr::watch_style;

        crate::style::STYLE_MAIN_FILE
            .set(main_css_path.as_ref().to_path_buf())
            .unwrap();

        watch_style();

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
