use hyro::{context, prelude::*};
use trillium::Conn;
use trillium_router::Router;

fn main() {
    hyro::config::set_template_file_extension("html.j2").unwrap();

    trillium_smol::config()
        .with_host("0.0.0.0")
        .with_port(1380)
        .with_nodelay()
        .without_signals()
        .run(Router::new().get("/", index).with_hmr())
}

async fn index(mut conn: Conn) -> Conn {
    let template = conn.template().await;
    conn.with_body(template.render(context!()).to_string())
}
