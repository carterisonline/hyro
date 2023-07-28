## 0.1.0

Initial commit

## 0.2.0

### Unreasonable speed improvement for plain templates

When setting up benchmarks, I noticed that large templates without any logic (essentially plain HTML)
took longer to render than a small template with only two inputs. Since, without HMR, templates aren't
modified during runtime, "rendering" plain templates will share a `&'static str`. We don't make any
calls to minijinja or clone data. The only API difference is that endpoints previously returning a
`Html<String>` should now return a `Html<Cow<'static, str>>`.

| Version   | Benchmark        | Average Speed        |
| --------- | ---------------- | -------------------- |
| 0.1.0     | form_and_context | 2.57µs (389K /s)     |
| 0.1.0     | plain            | 7.57µs (132K /s)     |
| **0.2.0** | form_and_context | **2.39µs (418K /s)** |
| **0.2.0** | plain            | **39ns (25.6M /s)**  |

### Minor changes

- Added `set_template_dir` for... setting the template directory. Defaults to `templates`.
- Fixed an issue where `cargo check --release` would fail because of a missing generic at `src/template.rs:57`.

## 0.2.1

### Minor changes

- Fixed an issue where changes to an `index` template wouldn't perform a full-reload and would render
  to an incorrect and unused endpoint.
