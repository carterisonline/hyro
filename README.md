<img alt="HYRO logo" src="./assets/hyro-dark.svg#gh-dark-mode-only" align="right" height="130" width="200"/>
<img alt="HYRO logo" src="./assets/hyro-light.svg#gh-light-mode-only" align="right" height="130" width="200">

### HYRO

noun  
/ˈhɪr.oʊ/

1. A : acronym for "Hypermedia Rust Orchestration"  
   B : a crate that extends existing frameworks like [Axum](https://github.com/tokio-rs/axum/) with new functionality, like
   rendering [Jinja Templates](https://github.com/mitsuhiko/minijinja) on the server,
   [bundling css](https://github.com/parcel-bundler/lightningcss), and a better developer experience.  
   C : a powerful HMR framework for [hypermedia systems](https://hypermedia.systems/) like [HTMX](https://htmx.org/).  
   D : the equivalent of [Rails](https://rubyonrails.org/) for nerds

## Usage and Examples

- More in-depth examples can be found in the [examples folder](examples/). Make sure you `cd` to the path containing
  the templates and style folders before running or _you will get a file-not-found error!_

Let's start with dependencies. We'll be using axum as our framework, and tokio as our runtime:

```sh
cargo new hyro-getting-started

cargo add hyro -F framework-axum
cargo add axum
cargo add tokio -F full
mkdir templates
```

HYRO templates use Jinja2. Let's start with a basic one:

`templates/hello.html.jinja2`

```html
<p>Hello, {{ name }}!</p>
```

Then we can set up our boilerplate:

`src/main.rs`

```rust
use std::borrow::Cow;

use axum::response::Html;
use axum::{routing, Router, Server};
use hyro::prelude::*;
use hyro::{context, Template};

#[tokio::main]
async fn main() {
   let router = Router::new()
      .route("/hello", routing::get(hello))
      .into_service_with_hmr();

   Server::from_tcp(hyro::bind("0.0.0.0:1380"))).unwrap()
        .serve(router)
        .await
        .unwrap();
}

async fn hello(template: Template) -> Html<Cow<'static, str>> {
   template.render(context! {
      name => "World",
   })
}
```

Now if we navigate to 'localhost:1380/hello', we can read our message! If you're running in
debug mode, you can edit `templates/hello.html.jinja2` and the HMR should kick in.
