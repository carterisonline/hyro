use axum::response::Html;
use axum::routing::get;

use hyro::style::{CARTERS_PARSER_OPTIONS, CARTERS_TARGET_OPTIONS};
use hyro::{context, RouterExt, Template};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let router = axum::Router::new()
        .route("/", get(index))
        .route("/hello", get(hello))
        .route("/navbar", get(navbar))
        .route("/splash", get(splash))
        .with_bundled_css(
            "/main.css",
            "style/.main.css",
            CARTERS_PARSER_OPTIONS.clone(),
            *CARTERS_TARGET_OPTIONS,
        )
        .nest_service("/assets", ServeDir::new("assets"))
        .into_service_with_hmr();

    axum::serve(hyro::bind("0.0.0.0:1380").await, router).await
}

async fn index(template: Template) -> Html<String> {
    template.render(context! {
        title => "Home",
    })
}

async fn navbar(template: Template) -> Html<String> {
    template.render(context!())
}

async fn hello(template: Template) -> Html<String> {
    template.render(context!())
}

async fn splash(template: Template) -> Html<String> {
    template.render(context!())
}
