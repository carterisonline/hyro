use std::collections::HashMap;

use criterion::*;
use hyro::{context, Template};
use tap::Tap;

pub fn render_benchmark(c: &mut Criterion) {
    hyro::config::set_template_dir("benches/templates").unwrap();
    c.bench_function("render_form_and_context", |b| {
        b.iter(|| {
            Template {
                path: "/form_and_context".into(),
                form: HashMap::new().tap_mut(|h| {
                    h.insert("name".to_string(), "world".to_string());
                }),
            }
            .render(context! {
                greeting => "Hello"
            })
        });
    });

    c.bench_function("render_plain", |b| {
        b.iter(|| {
            Template {
                path: "/plain".into(),
                form: HashMap::new(),
            }
            .render(context!())
        });
    });
}

criterion_group!(benches, render_benchmark);
criterion_main!(benches);
