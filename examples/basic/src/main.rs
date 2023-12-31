use std::borrow::Cow;

use axum::response::Html;
use axum::routing::get;

use hyro::prelude::*;
use hyro::{context, Template};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let router = axum::Router::new()
        .route("/", get(index))
        .route("/hello", get(hello))
        .route("/navbar", get(navbar))
        .route("/splash", get(splash))
        .with_bundled_css("/main.css", "style/.main.css")
        .nest_service("/assets", ServeDir::new("assets"))
        .into_service_with_hmr();

    axum::Server::from_tcp(hyro::bind("0.0.0.0:1380"))
        .unwrap()
        .serve(router)
        .await
        .unwrap();

    Ok(())
}

async fn index(template: Template) -> Html<Cow<'static, str>> {
    template.render(context! {
        title => "Home",
    })
}

async fn navbar(template: Template) -> Html<Cow<'static, str>> {
    template.render(context!())
}

async fn hello(template: Template) -> Html<Cow<'static, str>> {
    template.render(context!())
}

async fn splash(template: Template) -> Html<Cow<'static, str>> {
    template.render(context!())
}
